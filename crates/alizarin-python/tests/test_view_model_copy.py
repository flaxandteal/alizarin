"""
View models that subclass a builtin survive copy, deepcopy and pickle, and
DateViewModel keeps datetime's arithmetic (PARITY-RESEARCH.md F-5, F-6).
Their __new__ takes the stored value, where the copy protocols pass the
builtin's own arguments.
"""

import copy
import pickle
from datetime import timedelta

import pytest

from alizarin.static_types import StaticDomainValue, StaticValue
from alizarin.view_models import (
    ConceptValueViewModel,
    DateViewModel,
    DomainValueViewModel,
    EDTFViewModel,
    NonLocalizedStringViewModel,
    NumberViewModel,
    StringViewModel,
    Url,
    UrlViewModel,
)

VIEW_MODELS = {
    "string": lambda: StringViewModel({"en": {"value": "Alice", "direction": "ltr"}}),
    "non_localized_string": lambda: NonLocalizedStringViewModel("plain"),
    "edtf": lambda: EDTFViewModel("1984?"),
    "url": lambda: UrlViewModel(Url(url="https://example.org", url_label="Example")),
    "concept_value": lambda: ConceptValueViewModel(StaticValue(id="v1", value="Red"), "c1"),
    "domain_value": lambda: DomainValueViewModel(StaticDomainValue(id="d1", selected=False, text={"en": "Yes"})),
    "number": lambda: NumberViewModel(42.5),
    "date": lambda: DateViewModel("2024-03-01T12:30:00Z"),
}

COPIES = {
    "copy": copy.copy,
    "deepcopy": copy.deepcopy,
    "pickle": lambda vm: pickle.loads(pickle.dumps(vm)),
}


@pytest.mark.parametrize("how", sorted(COPIES))
@pytest.mark.parametrize("kind", sorted(VIEW_MODELS))
def test_copies_keep_type_and_value(kind, how):
    original = VIEW_MODELS[kind]()
    copied = COPIES[how](original)
    assert type(copied) is type(original)
    assert copied == original
    assert str(copied) == str(original)


@pytest.mark.parametrize("how", sorted(COPIES))
def test_copied_string_keeps_its_language_value(how):
    copied = COPIES[how](StringViewModel({"en": "Hello", "ga": "Dia duit"}, "ga"))
    assert str(copied) == "Dia duit"
    assert copied._value == {"en": "Hello", "ga": "Dia duit"}


def test_date_arithmetic_and_replace():
    date = DateViewModel("2024-03-01T12:30:00Z")
    assert (date + timedelta(days=1)).day == 2
    assert date.replace(year=2020).year == 2020
    assert date.astimezone().utcoffset() is not None
