![Alizarin: Fast JS Knowledge Graphs](./docs/banner.png)

# Alizarin

A high-performance TypeScript/JavaScript SDK for working with [Arches](https://www.archesproject.org/) data management systems. Alizarin provides a sophisticated object-relational mapping (ORM) layer that enables seamless interaction with Arches' graph-based data models.

This is a pure JS/TS implementation of [AORM](https://github.com/flaxandteal/arches-orm/) for front or backend.

## Features

- **Universal Compatibility**: Works in both browser and Node.js environments
- **Type-Safe**: Full TypeScript support with comprehensive type definitions
- **Lazy Loading**: Optimized data fetching with on-demand tile loading
- **Graph-Based ORM**: Intuitive API for working with Arches' complex graph structures
- **Multiple Data Sources**: Support for remote servers, static JSON files, and local data
- **Internationalization**: Built-in support for multilingual content
- **Schema Manipulation**: Programmatically create and modify graph schemas
- **Caching**: Intelligent caching for improved performance

## Goals

Alizarin aims to:

1. **Simplify Arches Development**: Provide an intuitive, developer-friendly interface to Arches' complex data models
2. **Enable Modern Web Applications**: Support building reactive, performant front-end applications with Arches data
3. **Maintain Flexibility**: Work with any Arches instance without requiring server-side modifications
4. **Ensure Type Safety**: Leverage TypeScript to catch errors early and improve developer experience
5. **Optimize Performance**: Minimize network requests and memory usage through intelligent caching and lazy loading

## Installation

```bash
npm install alizarin
```

## Quick Start

### Basic Usage

```typescript
import { ArchesClientRemote, GraphManager } from 'alizarin';

// Connect to an Arches instance
const client = new ArchesClientRemote('https://your-arches-instance.org');
const graphManager = new GraphManager(client);

// Load a resource model
const MonumentModel = await graphManager.loadGraph('Monument');

// Query all monuments
const monuments = await MonumentModel.all();

// Access monument data
for (const monument of monuments) {
  console.log(monument.name);
  console.log(monument.constructionDate);
  console.log(monument.heritage_status?.prefLabel);
}
```

### Working with Static Data

```typescript
import { ArchesClientRemoteStatic, GraphManager } from 'alizarin';

// Load from static JSON exports
const client = new ArchesClientRemoteStatic('/data/exports');
const graphManager = new GraphManager(client);

// Usage is identical to remote client
const PersonModel = await graphManager.loadGraph('Person');
```

### Creating Resources

```typescript
const PersonModel = await graphManager.loadGraph('Person');

// Create a new person
const newPerson = PersonModel.create();
newPerson.forename = "Jane";
newPerson.surname = "Smith";
newPerson.birthDate = new Date(1990, 5, 15);
```

## API Reference

### Client Classes

- `ArchesClient` - Abstract base class for all clients
- `ArchesClientRemote` - Connect to live Arches servers
- `ArchesClientRemoteStatic` - Load from static JSON files
- `ArchesClientLocal` - Load from local filesystem (Node.js)

### Core Classes

- `GraphManager` - Central manager for graphs and resources
- `ResourceModelWrapper` - ORM wrapper for resource models
- `ResourceInstanceViewModel` - View model for resource instances
- `GraphMutator` - Tool for modifying graph schemas

### View Model Types

- `StringViewModel` - String values
- `NumberViewModel` - Numeric values
- `DateViewModel` - Date/time values
- `ConceptValueViewModel` - Controlled vocabulary concepts
- `GeojsonFeatureCollectionViewModel` - Geographic data
- `ResourceInstanceListViewModel` - Related resources
- `SemanticViewModel` - Complex nested structures

## Development

### Running Tests

```bash
npm test
```

### Linting

```bash
npm run lint
```

### CI

To get linting up and running, this uses `typescript-eslint`. Currently, this has a
temporary allowed limit of 46 warnings, and sets several errors to warnings. This
is temporary and will be progressively reduced.

### License

Currently, and you should assume for the foreseeable future,
any use of this package, even as a front-end library, carries AGPL requirements and that
means any derived work must be licensed appropriately, with shared source-code. This, for
avoidance of doubt, means original untranspiled Javascript, Typescript etc. must be made
public to all web-users for the whole of any web platform using this library.

**This library unlikely to be suitable for use in most traditional commercial products.**

We may, in future, dual-license or relicense this package more liberally, so please note
that we will expect **any PRs to be MIT-licensed** to enable the possibility.
This may seem lopsided if we begin receiving PRs on the scale of the existing project,
so if you are considering doing a
substantial piece of work, get in touch beforehand to see if a relicense is possible (which
may depend on third-party discussions) and how we can handle it.

**Please note** that there is third-party code in the `tests/` subdirectory.

### Acknowledgments

Thanks to the folks at [Historic England](https://historicengland.org.uk/), the
[GCI](https://www.getty.edu/conservation/) and the [Arches Developer Community](https://www.archesproject.org/)
for the fantastic Arches project, and to the
[Historic Environment Division](https://www.communities-ni.gov.uk/topics/historic-environment) for their
support of our related Arches work.

In particular, the test data is based on the resource models from [Arches for HERs](https://www.archesproject.org/arches-for-hers/)
and [Arches for Science](https://www.archesproject.org/arches-for-science/).
