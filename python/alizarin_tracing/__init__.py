"""
Unified tracing infrastructure for Alizarin (Python)

This module provides a consistent tracing API that mirrors the JS/TS
implementation, making it easy to correlate traces across Python and
JavaScript/WASM consumers of Alizarin.

The API is designed to be compatible with OpenTelemetry semantics,
and can optionally use the real OpenTelemetry SDK if available.
"""

from __future__ import annotations

import time
import uuid
import threading
from dataclasses import dataclass, field
from typing import (
    Any,
    Callable,
    Dict,
    List,
    Optional,
    TypeVar,
    Union,
    ContextManager,
)
from contextlib import contextmanager
from functools import wraps

# Type aliases
SpanAttributes = Dict[str, Union[str, int, float, bool, None]]
SpanExporter = Callable[[List["SpanData"]], None]

T = TypeVar("T")


def _generate_id(length: int) -> str:
    """Generate a random hex ID."""
    return uuid.uuid4().hex[:length]


def _generate_trace_id() -> str:
    """Generate a 32-character trace ID."""
    return _generate_id(32)


def _generate_span_id() -> str:
    """Generate a 16-character span ID."""
    return _generate_id(16)


def _now_ms() -> float:
    """Get current time in milliseconds (high resolution)."""
    return time.perf_counter() * 1000


@dataclass
class SpanContext:
    """Context identifying a span within a trace."""
    trace_id: str
    span_id: str
    parent_span_id: Optional[str] = None


@dataclass
class SpanEvent:
    """An event that occurred during a span."""
    name: str
    timestamp: float
    attributes: SpanAttributes = field(default_factory=dict)


@dataclass
class SpanData:
    """Complete data for a finished span."""
    name: str
    context: SpanContext
    start_time: float
    end_time: Optional[float] = None
    duration: Optional[float] = None
    attributes: SpanAttributes = field(default_factory=dict)
    status: str = "unset"  # 'ok', 'error', 'unset'
    events: List[SpanEvent] = field(default_factory=list)


# Thread-local storage for active span stack
_span_stack = threading.local()


def _get_span_stack() -> List["Span"]:
    """Get the span stack for the current thread."""
    if not hasattr(_span_stack, "stack"):
        _span_stack.stack = []
    return _span_stack.stack


class Span:
    """
    A tracing span representing a unit of work.

    Can be used as a context manager:

        with tracer.start_span("my-operation") as span:
            span.set_attribute("key", "value")
            # do work...

    Or manually:

        span = tracer.start_span("my-operation")
        try:
            # do work...
            span.set_status("ok")
        except Exception as e:
            span.record_exception(e)
        finally:
            span.end()
    """

    def __init__(
        self,
        name: str,
        tracer: "Tracer",
        parent_context: Optional[SpanContext] = None,
        attributes: Optional[SpanAttributes] = None,
    ):
        self._tracer = tracer
        self._ended = False
        self._data = SpanData(
            name=name,
            context=SpanContext(
                trace_id=parent_context.trace_id if parent_context else _generate_trace_id(),
                span_id=_generate_span_id(),
                parent_span_id=parent_context.span_id if parent_context else None,
            ),
            start_time=_now_ms(),
            attributes=attributes.copy() if attributes else {},
        )

    @property
    def context(self) -> SpanContext:
        """Get the span's context."""
        return self._data.context

    @property
    def name(self) -> str:
        """Get the span's name."""
        return self._data.name

    def set_attribute(self, key: str, value: Union[str, int, float, bool]) -> "Span":
        """Set a single attribute on the span."""
        if not self._ended:
            self._data.attributes[key] = value
        return self

    def set_attributes(self, attributes: SpanAttributes) -> "Span":
        """Set multiple attributes on the span."""
        if not self._ended:
            self._data.attributes.update(attributes)
        return self

    def add_event(
        self, name: str, attributes: Optional[SpanAttributes] = None
    ) -> "Span":
        """Add an event to the span."""
        if not self._ended:
            self._data.events.append(
                SpanEvent(
                    name=name,
                    timestamp=_now_ms(),
                    attributes=attributes or {},
                )
            )
        return self

    def set_status(self, status: str, message: Optional[str] = None) -> "Span":
        """Set the span's status ('ok' or 'error')."""
        if not self._ended:
            self._data.status = status
            if message:
                self._data.attributes["status.message"] = message
        return self

    def record_exception(self, exception: BaseException) -> "Span":
        """Record an exception on the span."""
        if not self._ended:
            import traceback

            self.add_event(
                "exception",
                {
                    "exception.type": type(exception).__name__,
                    "exception.message": str(exception),
                    "exception.stacktrace": "".join(
                        traceback.format_exception(
                            type(exception), exception, exception.__traceback__
                        )
                    ),
                },
            )
            self.set_status("error", str(exception))
        return self

    def end(self) -> None:
        """End the span and report it to the tracer."""
        if self._ended:
            return

        self._ended = True
        self._data.end_time = _now_ms()
        self._data.duration = self._data.end_time - self._data.start_time

        # Remove from stack if it's the current span
        stack = _get_span_stack()
        if stack and stack[-1] is self:
            stack.pop()

        # Report to tracer
        self._tracer._on_span_end(self._data)

    def get_data(self) -> SpanData:
        """Get a copy of the span's data."""
        return SpanData(
            name=self._data.name,
            context=SpanContext(
                trace_id=self._data.context.trace_id,
                span_id=self._data.context.span_id,
                parent_span_id=self._data.context.parent_span_id,
            ),
            start_time=self._data.start_time,
            end_time=self._data.end_time,
            duration=self._data.duration,
            attributes=self._data.attributes.copy(),
            status=self._data.status,
            events=self._data.events.copy(),
        )

    def __enter__(self) -> "Span":
        _get_span_stack().append(self)
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        if exc_val is not None:
            self.record_exception(exc_val)
        elif self._data.status == "unset":
            self.set_status("ok")
        self.end()


class Tracer:
    """
    Tracer creates spans and manages their export.

    Usage:
        tracer = get_tracer("my-service")
        with tracer.start_span("operation") as span:
            span.set_attribute("key", "value")
            # do work...
    """

    def __init__(self, name: str, version: Optional[str] = None):
        self._name = name
        self._version = version
        self._exporters: List[SpanExporter] = []
        self._pending_spans: List[SpanData] = []
        self._batch_size = 100
        self._lock = threading.Lock()

    def add_exporter(self, exporter: SpanExporter) -> "Tracer":
        """Add an exporter to receive completed spans."""
        self._exporters.append(exporter)
        return self

    def start_span(
        self,
        name: str,
        attributes: Optional[SpanAttributes] = None,
    ) -> Span:
        """Start a new span."""
        stack = _get_span_stack()
        parent_context = stack[-1].context if stack else None
        return Span(name, self, parent_context, attributes)

    @contextmanager
    def start_as_current_span(
        self,
        name: str,
        attributes: Optional[SpanAttributes] = None,
    ) -> ContextManager[Span]:
        """Start a span and make it the current span in context."""
        span = self.start_span(name, attributes)
        with span:
            yield span

    def _on_span_end(self, span_data: SpanData) -> None:
        """Called when a span ends."""
        # Add tracer info
        span_data.attributes["tracer.name"] = self._name
        if self._version:
            span_data.attributes["tracer.version"] = self._version

        with self._lock:
            self._pending_spans.append(span_data)
            if len(self._pending_spans) >= self._batch_size:
                self._flush_locked()

    def flush(self) -> None:
        """Flush pending spans to exporters."""
        with self._lock:
            self._flush_locked()

    def _flush_locked(self) -> None:
        """Flush while holding the lock."""
        if not self._pending_spans:
            return

        spans = self._pending_spans
        self._pending_spans = []

        for exporter in self._exporters:
            try:
                exporter(spans)
            except Exception as e:
                print(f"Exporter error: {e}")

    def get_current_span(self) -> Optional[Span]:
        """Get the current active span."""
        stack = _get_span_stack()
        return stack[-1] if stack else None


class TracerProvider:
    """Manages tracer instances."""

    def __init__(self):
        self._tracers: Dict[str, Tracer] = {}
        self._global_exporters: List[SpanExporter] = []
        self._lock = threading.Lock()

    def get_tracer(self, name: str, version: Optional[str] = None) -> Tracer:
        """Get or create a tracer instance."""
        key = f"{name}@{version or 'unknown'}"

        with self._lock:
            if key not in self._tracers:
                tracer = Tracer(name, version)
                for exporter in self._global_exporters:
                    tracer.add_exporter(exporter)
                self._tracers[key] = tracer

            return self._tracers[key]

    def add_global_exporter(self, exporter: SpanExporter) -> "TracerProvider":
        """Add an exporter to all current and future tracers."""
        with self._lock:
            self._global_exporters.append(exporter)
            for tracer in self._tracers.values():
                tracer.add_exporter(exporter)
        return self

    def flush_all(self) -> None:
        """Flush all tracers."""
        with self._lock:
            tracers = list(self._tracers.values())
        for tracer in tracers:
            tracer.flush()


# Global tracer provider
_global_provider = TracerProvider()


def get_tracer(name: str, version: Optional[str] = None) -> Tracer:
    """Get a tracer instance from the global provider."""
    return _global_provider.get_tracer(name, version)


def add_global_exporter(exporter: SpanExporter) -> None:
    """Add an exporter to all tracers."""
    _global_provider.add_global_exporter(exporter)


def flush_all() -> None:
    """Flush all pending spans."""
    _global_provider.flush_all()


# ============================================================================
# Built-in Exporters
# ============================================================================


def console_exporter(spans: List[SpanData]) -> None:
    """Log spans to console."""
    print("=== Trace Spans ===")
    for span in spans:
        indent = "  " if span.context.parent_span_id else ""
        status_str = f"({span.status})" if span.status != "unset" else ""
        duration_str = f"{span.duration:.2f}ms" if span.duration else "?"
        print(f"{indent}[{span.name}] {duration_str} {status_str}")


class SummaryExporter:
    """Aggregates timing stats like the old system."""

    def __init__(self):
        self._stats: Dict[str, Dict[str, float]] = {}
        self._lock = threading.Lock()

    def __call__(self, spans: List[SpanData]) -> None:
        """Export spans (called by tracer)."""
        with self._lock:
            for span in spans:
                duration = span.duration or 0
                if span.name not in self._stats:
                    self._stats[span.name] = {
                        "count": 0,
                        "total_ms": 0,
                        "min_ms": float("inf"),
                        "max_ms": 0,
                    }

                stats = self._stats[span.name]
                stats["count"] += 1
                stats["total_ms"] += duration
                stats["min_ms"] = min(stats["min_ms"], duration)
                stats["max_ms"] = max(stats["max_ms"], duration)

    def get_summary(self) -> Dict[str, Dict[str, float]]:
        """Get the timing summary."""
        with self._lock:
            result = {}
            for name, stats in self._stats.items():
                count = stats["count"]
                result[name] = {
                    **stats,
                    "avg_ms": stats["total_ms"] / count if count > 0 else 0,
                }
            return result

    def print_summary(self, label: str = "") -> None:
        """Print the timing summary."""
        print(f"=== Timing Summary {label} ===")
        summary = self.get_summary()
        entries = sorted(summary.items(), key=lambda x: -x[1]["total_ms"])

        for name, stats in entries:
            print(
                f"{name}: count={stats['count']:.0f}, "
                f"total={stats['total_ms']:.2f}ms, "
                f"avg={stats['avg_ms']:.2f}ms, "
                f"min={stats['min_ms']:.2f}ms, "
                f"max={stats['max_ms']:.2f}ms"
            )

    def reset(self) -> None:
        """Reset all stats."""
        with self._lock:
            self._stats.clear()


# ============================================================================
# Convenience decorators
# ============================================================================


def traced(
    name: Optional[str] = None,
    tracer: Optional[Tracer] = None,
    attributes: Optional[SpanAttributes] = None,
):
    """
    Decorator to trace a function.

    Usage:
        @traced("my-operation")
        def my_function():
            ...

        @traced()  # Uses function name
        def another_function():
            ...
    """

    def decorator(func: Callable[..., T]) -> Callable[..., T]:
        span_name = name or func.__name__
        t = tracer or get_tracer("alizarin")

        @wraps(func)
        def wrapper(*args, **kwargs) -> T:
            with t.start_as_current_span(span_name, attributes) as span:
                span.set_attribute("args.count", len(args))
                return func(*args, **kwargs)

        return wrapper

    return decorator


# ============================================================================
# OpenTelemetry compatibility layer
# ============================================================================


def try_use_opentelemetry() -> bool:
    """
    Try to use real OpenTelemetry if available.

    Returns True if OpenTelemetry is being used, False if using built-in.
    """
    try:
        from opentelemetry import trace as otel_trace
        from opentelemetry.sdk.trace import TracerProvider as OtelTracerProvider
        from opentelemetry.sdk.trace.export import (
            BatchSpanProcessor,
            ConsoleSpanExporter,
        )

        # Set up OpenTelemetry
        provider = OtelTracerProvider()
        processor = BatchSpanProcessor(ConsoleSpanExporter())
        provider.add_span_processor(processor)
        otel_trace.set_tracer_provider(provider)

        print("[alizarin-tracing] Using OpenTelemetry SDK")
        return True
    except ImportError:
        print("[alizarin-tracing] OpenTelemetry not available, using built-in tracing")
        return False
