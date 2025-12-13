"""
Alizarin CLI - Convert JSON trees to Arches tile data.

Usage:
    alizarin convert --graph model.json --input trees.json --output tiles.json
"""

import json
import sys
import uuid
from pathlib import Path
from typing import Optional

import click

from . import json_tree_to_tiles, batch_trees_to_tiles, batch_tiles_to_trees


@click.group()
@click.version_option()
def cli():
    """Alizarin - Tools for working with Arches data."""
    pass


@cli.command()
@click.option(
    '--graph', '-g',
    type=click.Path(exists=True, path_type=Path),
    required=True,
    help='Path to the graph/model JSON file.'
)
@click.option(
    '--input', '-i',
    type=click.Path(exists=True, path_type=Path),
    required=True,
    help='Path to input JSON file containing an array of JSON trees.'
)
@click.option(
    '--output', '-o',
    type=click.Path(path_type=Path),
    required=True,
    help='Path to output JSON file for tile data.'
)
@click.option(
    '--graph-id',
    type=str,
    default=None,
    help='Override graph ID (defaults to graphid from graph file).'
)
@click.option(
    '--pretty/--compact',
    default=True,
    help='Pretty-print output JSON (default: pretty).'
)
@click.option(
    '--from-camel',
    is_flag=True,
    default=False,
    help='Convert keys from camelCase to snake_case before resolving with graph.'
)
def convert(
    graph: Path,
    input: Path,
    output: Path,
    graph_id: Optional[str],
    pretty: bool,
    from_camel: bool
):
    """
    Convert JSON trees to Arches tile data.

    Takes an array of JSON tree structures and converts them to tile format
    suitable for import into Arches.

    Example:
        alizarin convert -g Person.json -i people.json -o tiles.json
    """
    if json_tree_to_tiles is None:
        click.echo("Error: Rust extension not available. Please rebuild alizarin.", err=True)
        sys.exit(1)

    # Load graph
    try:
        with open(graph, 'r', encoding='utf-8') as f:
            graph_data = json.load(f)
    except json.JSONDecodeError as e:
        click.echo(f"Error: Invalid JSON in graph file: {e}", err=True)
        sys.exit(1)

    # Extract graph ID
    if graph_id is None:
        # Try to find graphid in the graph file
        if 'graphid' in graph_data:
            graph_id = graph_data['graphid']
        elif 'graph' in graph_data and isinstance(graph_data['graph'], list):
            # Arches export format: {"graph": [{"graphid": "..."}]}
            graph_id = graph_data['graph'][0].get('graphid')

        if graph_id is None:
            click.echo("Error: Could not find graphid in graph file. Use --graph-id to specify.", err=True)
            sys.exit(1)

    # Normalize graph data for the Rust function
    # If it's in Arches export format, extract the inner graph
    if 'graph' in graph_data and isinstance(graph_data['graph'], list):
        inner_graph = graph_data['graph'][0]
    else:
        inner_graph = graph_data

    graph_json = json.dumps(inner_graph)

    # Load input trees
    try:
        with open(input, 'r', encoding='utf-8') as f:
            trees = json.load(f)
    except json.JSONDecodeError as e:
        click.echo(f"Error: Invalid JSON in input file: {e}", err=True)
        sys.exit(1)

    if not isinstance(trees, list):
        click.echo("Error: Input file must contain a JSON array of trees.", err=True)
        sys.exit(1)

    # Use batch conversion for parallel processing
    click.echo(f"Converting {len(trees)} trees (parallel)...")

    try:
        trees_json = json.dumps(trees)
        batch_result = batch_trees_to_tiles(
            trees_json=trees_json,
            graph_json=graph_json,
            from_camel=from_camel
        )

        results = batch_result.get('results', [])
        errors = batch_result.get('errors', [])
        error_count = batch_result.get('error_count', 0)

    except Exception as e:
        click.echo(f"Error during batch conversion: {e}", err=True)
        sys.exit(1)

    # Report results
    click.echo(f"\nConverted {len(results)} trees successfully.")

    if errors:
        click.echo(f"Failed to convert {len(errors)} trees:", err=True)
        for err in errors[:5]:  # Show first 5 errors
            click.echo(f"  {err}", err=True)
        if len(errors) > 5:
            click.echo(f"  ... and {len(errors) - 5} more errors", err=True)

    # Write output
    try:
        with open(output, 'w', encoding='utf-8') as f:
            if pretty:
                json.dump(results, f, indent=2, ensure_ascii=False)
            else:
                json.dump(results, f, ensure_ascii=False)
        click.echo(f"Output written to {output}")
    except IOError as e:
        click.echo(f"Error writing output file: {e}", err=True)
        sys.exit(1)

    # Exit with error if any conversions failed
    if errors:
        sys.exit(1)


@cli.command()
@click.option(
    '--graph', '-g',
    type=click.Path(exists=True, path_type=Path),
    required=True,
    help='Path to the graph/model JSON file.'
)
@click.option(
    '--input', '-i',
    type=click.Path(exists=True, path_type=Path),
    required=True,
    help='Path to input JSON file containing resources with tiles.'
)
@click.option(
    '--output', '-o',
    type=click.Path(path_type=Path),
    required=True,
    help='Path to output JSON file for JSON trees.'
)
@click.option(
    '--pretty/--compact',
    default=True,
    help='Pretty-print output JSON (default: pretty).'
)
def to_trees(
    graph: Path,
    input: Path,
    output: Path,
    pretty: bool
):
    """
    Convert Arches tile data to JSON trees.

    Takes an array of resources with tiles and converts them to nested JSON
    tree structures.

    Example:
        alizarin to-trees -g Person.json -i resources.json -o trees.json
    """
    from . import tiles_to_json_tree

    if tiles_to_json_tree is None:
        click.echo("Error: Rust extension not available. Please rebuild alizarin.", err=True)
        sys.exit(1)

    # Load graph
    try:
        with open(graph, 'r', encoding='utf-8') as f:
            graph_data = json.load(f)
    except json.JSONDecodeError as e:
        click.echo(f"Error: Invalid JSON in graph file: {e}", err=True)
        sys.exit(1)

    # Extract graph ID and normalize
    if 'graph' in graph_data and isinstance(graph_data['graph'], list):
        inner_graph = graph_data['graph'][0]
        graph_id = inner_graph.get('graphid')
    else:
        inner_graph = graph_data
        graph_id = graph_data.get('graphid')

    if graph_id is None:
        click.echo("Error: Could not find graphid in graph file.", err=True)
        sys.exit(1)

    graph_json = json.dumps(inner_graph)

    # Load input resources
    try:
        with open(input, 'r', encoding='utf-8') as f:
            resources = json.load(f)
    except json.JSONDecodeError as e:
        click.echo(f"Error: Invalid JSON in input file: {e}", err=True)
        sys.exit(1)

    if not isinstance(resources, list):
        resources = [resources]

    # Use batch conversion for parallel processing
    click.echo(f"Converting {len(resources)} resources to trees (parallel)...")

    try:
        resources_json = json.dumps(resources)
        batch_result = batch_tiles_to_trees(
            resources_json=resources_json,
            graph_json=graph_json
        )

        results = batch_result.get('results', [])
        errors = batch_result.get('errors', [])

    except Exception as e:
        click.echo(f"Error during batch conversion: {e}", err=True)
        sys.exit(1)

    # Report results
    click.echo(f"\nConverted {len(results)} resources successfully.")

    if errors:
        click.echo(f"Failed to convert {len(errors)} resources:", err=True)
        for err in errors[:5]:
            click.echo(f"  {err}", err=True)

    # Write output
    try:
        with open(output, 'w', encoding='utf-8') as f:
            if pretty:
                json.dump(results, f, indent=2, ensure_ascii=False)
            else:
                json.dump(results, f, ensure_ascii=False)
        click.echo(f"Output written to {output}")
    except IOError as e:
        click.echo(f"Error writing output file: {e}", err=True)
        sys.exit(1)

    if errors:
        sys.exit(1)


@cli.command()
@click.option(
    '--graph', '-g',
    type=click.Path(exists=True, path_type=Path),
    required=True,
    help='Path to the graph/model JSON file.'
)
def info(graph: Path):
    """
    Display information about a graph/model file.

    Example:
        alizarin info -g Person.json
    """
    try:
        with open(graph, 'r', encoding='utf-8') as f:
            graph_data = json.load(f)
    except json.JSONDecodeError as e:
        click.echo(f"Error: Invalid JSON in graph file: {e}", err=True)
        sys.exit(1)

    # Handle Arches export format
    if 'graph' in graph_data and isinstance(graph_data['graph'], list):
        inner_graph = graph_data['graph'][0]
    else:
        inner_graph = graph_data

    click.echo(f"Graph ID: {inner_graph.get('graphid', '(unknown)')}")

    name = inner_graph.get('name', {})
    if isinstance(name, dict):
        name_str = name.get('en', name.get(list(name.keys())[0] if name else '', '(unnamed)'))
    else:
        name_str = str(name)
    click.echo(f"Name: {name_str}")

    nodes = inner_graph.get('nodes', [])
    click.echo(f"Nodes: {len(nodes)}")

    # Count datatypes
    datatypes = {}
    for node in nodes:
        dt = node.get('datatype', 'none')
        datatypes[dt] = datatypes.get(dt, 0) + 1

    if datatypes:
        click.echo("Datatypes:")
        for dt, count in sorted(datatypes.items(), key=lambda x: -x[1]):
            click.echo(f"  {dt}: {count}")


def main():
    """Entry point for the CLI."""
    cli()


if __name__ == '__main__':
    main()
