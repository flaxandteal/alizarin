"""Test AttrPromise for chainable lazy loading."""

import pytest
from alizarin.view_models import AttrPromise


class MockObject:
    """Mock object with nested attributes for testing."""
    def __init__(self):
        self.foo = MockNested()


class MockNested:
    """Mock nested object."""
    def __init__(self):
        self.bar = "baz"


@pytest.mark.asyncio
async def test_attr_promise_simple_await():
    """Test that AttrPromise can be awaited."""
    async def get_value():
        return "test_value"

    promise = AttrPromise(get_value())
    result = await promise

    assert result == "test_value"


@pytest.mark.asyncio
async def test_attr_promise_chained_access():
    """Test that AttrPromise supports chained property access."""
    async def get_object():
        return MockObject()

    # This should work: await promise.foo.bar
    promise = AttrPromise(get_object())
    result = await promise.foo.bar

    assert result == "baz"


@pytest.mark.asyncio
async def test_attr_promise_indexing():
    """Test that AttrPromise supports indexing."""
    async def get_list():
        return [MockObject(), MockObject()]

    # This should work: await promise[0].foo.bar
    promise = AttrPromise(get_list())
    result = await promise[0].foo.bar

    assert result == "baz"


@pytest.mark.asyncio
async def test_attr_promise_none_safe():
    """Test that AttrPromise handles None values gracefully."""
    async def get_none():
        return None

    promise = AttrPromise(get_none())
    result = await promise.foo.bar

    assert result is None


@pytest.mark.asyncio
async def test_attr_promise_full_chain():
    """Test a realistic chaining scenario similar to group.member[0].names.name"""

    class Name:
        def __init__(self):
            self.forename = "Alice"
            self.surname = "Smith"

    class Names:
        def __init__(self):
            self.name = [Name()]

    class Member:
        def __init__(self):
            self.names = Names()

    class Group:
        def __init__(self):
            self.member = [Member()]

    async def get_group():
        return Group()

    # Simulate: await group.member[0].names.name[0].forename
    promise = AttrPromise(get_group())
    result = await promise.member[0].names.name[0].forename

    assert result == "Alice"


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
