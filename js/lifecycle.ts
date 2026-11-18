/**
 * Resource instance lifecycles (Arches 8+): resolve a resource's state and
 * move it between states. Both run in core on either backend; see
 * crates/alizarin-core/src/graph/lifecycle.rs for the rules.
 *
 * `lifecycle` is the graph's `resource_instance_lifecycle` (`null` when the
 * graph has none: a single Perpetual state that cannot be left).
 */
import {
  resourceLifecycleState as wasmResourceLifecycleState,
  transitionResourceLifecycle as wasmTransitionResourceLifecycle,
} from '../pkg/alizarin';
import { getBackend, getNapiModule, safeStringify } from './backend';

/** A resource's resolved lifecycle state. Every field is always present. */
export interface LifecycleStateReport {
  /** `state`, `unknown` (an id the lifecycle doesn't define) or `conflict` (layers disagreed). */
  status: 'state' | 'unknown' | 'conflict';
  /** `null` for a conflict and for the Perpetual state of a graph without a lifecycle. */
  state_id: string | null;
  is_initial: boolean;
  /** The ids the layers disagreed on; empty unless `status` is `conflict`. */
  conflict: string[];
}

/** One state change, for the caller to log or persist. */
export interface TransitionEvent {
  resourceinstanceid: string;
  /** The state left; `null` when leaving a conflict. */
  from: string | null;
  to: string;
  /** Present when the transition resolved a conflict. */
  resolved_conflict?: string[];
}

/** A tiles-free overlay record to compose over the resource, plus the event. */
export interface TransitionResult {
  resource: unknown;
  event: TransitionEvent;
}

/**
 * Resolve `resource`'s lifecycle state. An absent stored state is the
 * lifecycle's initial state; an undefined id is reported, never guessed.
 */
export function resourceLifecycleState(resource: unknown, lifecycle: unknown): LifecycleStateReport {
  if (getBackend() === 'napi') {
    const napi = getNapiModule();
    if (napi?.resourceLifecycleState) {
      return napi.resourceLifecycleState(safeStringify(resource), safeStringify(lifecycle ?? null));
    }
  }
  return wasmResourceLifecycleState(
    safeStringify(resource),
    safeStringify(lifecycle ?? null),
  ) as LifecycleStateReport;
}

/**
 * Move `resource` to the state `target`. Returns a tiles-free overlay record
 * (compose it over the resource as the topmost layer) and the event; nothing
 * is persisted.
 *
 * `strict` defaults to `true`: the move must follow one of the lifecycle's
 * next/previous edges, and a conflicted or unknown state cannot be left. Pass
 * `false` explicitly for Arches' own check, which is only that `target`
 * belongs to the lifecycle.
 */
export function transitionResourceLifecycle(
  resource: unknown,
  lifecycle: unknown,
  target: string,
  strict: boolean = true,
): TransitionResult {
  if (getBackend() === 'napi') {
    const napi = getNapiModule();
    if (napi?.transitionResourceLifecycle) {
      return napi.transitionResourceLifecycle(
        safeStringify(resource),
        safeStringify(lifecycle ?? null),
        target,
        strict,
      );
    }
  }
  return wasmTransitionResourceLifecycle(
    safeStringify(resource),
    safeStringify(lifecycle ?? null),
    target,
    strict,
  ) as TransitionResult;
}
