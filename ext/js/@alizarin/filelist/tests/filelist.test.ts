import { describe, test, assert, beforeAll } from 'vitest';

describe("FileList Extension", () => {
  beforeAll(async () => {
    // Initialize WASM via alizarin's wasmReady promise
    const { wasmReady } = await import("alizarin");
    await wasmReady;
  });

  // ===========================================================================
  // Extension Registration Tests
  // ===========================================================================

  describe("Extension Registration", () => {
    test("FileList extension module can be imported and registers file-list datatype", async () => {
      // Import the FileList extension which should register the datatype
      await import("../src/main");

      // Import viewModels from alizarin
      const alizarinModule = await import("alizarin");
      const CUSTOM_DATATYPES = alizarinModule.viewModels.CUSTOM_DATATYPES;

      // Verify the file-list datatype is now registered
      const fileListDatatype = CUSTOM_DATATYPES.get("file-list");
      assert.isDefined(fileListDatatype, "file-list datatype should be registered by extension");
      assert.isDefined(fileListDatatype.__create, "file-list datatype should have __create method");
    });

    test("Module exports expected items", async () => {
      const module = await import("../src/main");

      // Static types
      assert.isDefined(module.FileListItem, "Should export FileListItem");

      // View models
      assert.isDefined(module.FileItemViewModel, "Should export FileItemViewModel");
      assert.isDefined(module.FileListViewModel, "Should export FileListViewModel");
      assert.isDefined(module.FileListDataType, "Should export FileListDataType");
    });
  });

  // ===========================================================================
  // FileListItem Tests
  // ===========================================================================

  describe("FileListItem", () => {
    test("should create with minimal required fields", async () => {
      const { FileListItem } = await import("../src/main");

      const item = new FileListItem({
        name: "test.pdf",
      });

      assert.equal(item.name, "test.pdf");
      assert.equal(item.accepted, false);
      assert.equal(item.selected, false);
      assert.isUndefined(item.url);
      assert.isUndefined(item.file_id);
    });

    test("should create with all fields", async () => {
      const { FileListItem } = await import("../src/main");

      const item = new FileListItem({
        name: "photo.jpg",
        url: "/files/xyz789",
        file_id: "file-uuid-123",
        type: "image/jpeg",
        size: 12345,
        title: { en: { direction: "ltr", value: "My Photo" } },
        altText: { en: { direction: "ltr", value: "A beautiful sunset" } },
        accepted: true,
        selected: true,
      });

      assert.equal(item.name, "photo.jpg");
      assert.equal(item.url, "/files/xyz789");
      assert.equal(item.file_id, "file-uuid-123");
      assert.equal(item.type, "image/jpeg");
      assert.equal(item.size, 12345);
      assert.equal(item.accepted, true);
      assert.equal(item.selected, true);
    });

    test("should identify image files correctly", async () => {
      const { FileListItem } = await import("../src/main");

      const imageTypes = [
        "image/jpeg",
        "image/png",
        "image/gif",
        "image/webp",
        "image/svg+xml",
      ];

      for (const fileType of imageTypes) {
        const item = new FileListItem({ name: "test", type: fileType });
        assert.isTrue(item.isImage(), `${fileType} should be identified as image`);
      }

      const nonImageTypes = [
        "application/pdf",
        "text/plain",
        "video/mp4",
        "audio/mpeg",
        undefined,
      ];

      for (const fileType of nonImageTypes) {
        const item = new FileListItem({ name: "test", type: fileType });
        assert.isFalse(item.isImage(), `${fileType} should not be identified as image`);
      }
    });

    test("should return display string from title or name", async () => {
      const { FileListItem } = await import("../src/main");

      // With title
      const itemWithTitle = new FileListItem({
        name: "photo.jpg",
        title: { en: { direction: "ltr", value: "My Photo Title" } },
      });
      assert.equal(itemWithTitle.toDisplayString("en"), "My Photo Title");

      // Without title, falls back to name
      const itemWithoutTitle = new FileListItem({
        name: "document.pdf",
      });
      assert.equal(itemWithoutTitle.toDisplayString(), "document.pdf");
    });

    test("should get alt text for specific language", async () => {
      const { FileListItem } = await import("../src/main");

      const item = new FileListItem({
        name: "photo.jpg",
        altText: {
          en: { direction: "ltr", value: "English description" },
          es: { direction: "ltr", value: "Spanish description" },
        },
      });

      assert.equal(item.getAltText("en"), "English description");
      assert.equal(item.getAltText("es"), "Spanish description");
      assert.isNull(item.getAltText("de")); // Not found
    });

    test("should serialize to JSON correctly", async () => {
      const { FileListItem } = await import("../src/main");

      const item = new FileListItem({
        name: "test.pdf",
        url: "/files/test",
        file_id: "uuid-123",
        type: "application/pdf",
        size: 54321,
      });

      const json = item.toJson();

      assert.equal(json.name, "test.pdf");
      assert.equal(json.url, "/files/test");
      assert.equal(json.file_id, "uuid-123");
      assert.equal(json.type, "application/pdf");
      assert.equal(json.size, 54321);
    });
  });

  // ===========================================================================
  // FileItemViewModel Tests
  // ===========================================================================

  describe("FileItemViewModel", () => {
    test("should create from FileListItem", async () => {
      const { FileListItem, FileItemViewModel } = await import("../src/main");

      const item = new FileListItem({
        name: "document.pdf",
        url: "/files/doc",
        file_id: "doc-123",
        type: "application/pdf",
        size: 54321,
      });

      const vm = new FileItemViewModel(item);

      assert.equal(vm.name, "document.pdf");
      assert.equal(vm.url, "/files/doc");
      assert.equal(vm.file_id, "doc-123");
      assert.equal(vm.fileType, "application/pdf");
      assert.equal(vm.size, 54321);
    });

    test("should return name when converted to string", async () => {
      const { FileListItem, FileItemViewModel } = await import("../src/main");

      const item = new FileListItem({ name: "report.xlsx" });
      const vm = new FileItemViewModel(item);

      assert.equal(String(vm), "report.xlsx");
    });

    test("should delegate isImage to underlying item", async () => {
      const { FileListItem, FileItemViewModel } = await import("../src/main");

      const imageItem = new FileListItem({ name: "photo.jpg", type: "image/jpeg" });
      const pdfItem = new FileListItem({ name: "doc.pdf", type: "application/pdf" });

      assert.isTrue(new FileItemViewModel(imageItem).isImage());
      assert.isFalse(new FileItemViewModel(pdfItem).isImage());
    });

    test("should return underlying FileListItem via getValue", async () => {
      const { FileListItem, FileItemViewModel } = await import("../src/main");

      const item = new FileListItem({ name: "test.txt" });
      const vm = new FileItemViewModel(item);

      assert.strictEqual(vm.getValue(), item);
    });

    test("should serialize via forJson", async () => {
      const { FileListItem, FileItemViewModel } = await import("../src/main");

      const item = new FileListItem({
        name: "file.pdf",
        url: "/files/1",
        type: "application/pdf",
      });
      const vm = new FileItemViewModel(item);

      const json = await vm.forJson();

      assert.equal(json.name, "file.pdf");
      assert.equal(json.url, "/files/1");
      assert.equal(json.type, "application/pdf");
    });
  });

  // ===========================================================================
  // FileListViewModel Tests
  // ===========================================================================

  describe("FileListViewModel", () => {
    test("should create from array of file data", async () => {
      const { FileListViewModel } = await import("../src/main");
      const alizarin = await import("alizarin");

      const node = new alizarin.staticTypes.StaticNode({
        nodeid: "file-node-1",
        name: "Attachments",
        datatype: "file-list",
        nodegroup_id: "ng-1",
        alias: "attachments",
        graph_id: "test-graph",
        is_collector: true,
        isrequired: false,
        exportable: true,
        config: {},
      });

      const tile = new alizarin.staticTypes.StaticTile({
        tileid: "tile-1",
        nodegroup_id: "ng-1",
        resourceinstance_id: "resource-1",
        data: new Map(),
      });

      const fileData = [
        { name: "file1.pdf", url: "/files/1", file_id: "id-1", type: "application/pdf" },
        { name: "file2.jpg", url: "/files/2", file_id: "id-2", type: "image/jpeg" },
      ];

      const vm = await FileListViewModel.__create(tile, node, fileData);

      assert.isDefined(vm);
      assert.equal(vm.length, 2);
    });

    test("should handle empty file list", async () => {
      const { FileListViewModel } = await import("../src/main");
      const alizarin = await import("alizarin");

      const node = new alizarin.staticTypes.StaticNode({
        nodeid: "file-node-1",
        name: "Attachments",
        datatype: "file-list",
        nodegroup_id: "ng-1",
        alias: "attachments",
        graph_id: "test-graph",
        is_collector: true,
        isrequired: false,
        exportable: true,
        config: {},
      });

      const tile = new alizarin.staticTypes.StaticTile({
        tileid: "tile-1",
        nodegroup_id: "ng-1",
        resourceinstance_id: "resource-1",
        data: new Map(),
      });

      const vm = await FileListViewModel.__create(tile, node, []);

      assert.isDefined(vm);
      assert.equal(vm.length, 0);
    });

    test("should handle null data", async () => {
      const { FileListViewModel } = await import("../src/main");
      const alizarin = await import("alizarin");

      const node = new alizarin.staticTypes.StaticNode({
        nodeid: "file-node-1",
        name: "Attachments",
        datatype: "file-list",
        nodegroup_id: "ng-1",
        alias: "attachments",
        graph_id: "test-graph",
        is_collector: true,
        isrequired: false,
        exportable: true,
        config: {},
      });

      const tile = new alizarin.staticTypes.StaticTile({
        tileid: "tile-1",
        nodegroup_id: "ng-1",
        resourceinstance_id: "resource-1",
        data: new Map(),
      });

      const vm = await FileListViewModel.__create(tile, node, null);

      assert.isDefined(vm);
      assert.equal(vm.length, 0);
    });

    test("should filter images via getImages", async () => {
      const { FileListViewModel } = await import("../src/main");
      const alizarin = await import("alizarin");

      const node = new alizarin.staticTypes.StaticNode({
        nodeid: "file-node-1",
        name: "Attachments",
        datatype: "file-list",
        nodegroup_id: "ng-1",
        alias: "attachments",
        graph_id: "test-graph",
        is_collector: true,
        isrequired: false,
        exportable: true,
        config: {},
      });

      const tile = new alizarin.staticTypes.StaticTile({
        tileid: "tile-1",
        nodegroup_id: "ng-1",
        resourceinstance_id: "resource-1",
        data: new Map(),
      });

      const fileData = [
        { name: "doc.pdf", url: "/files/1", type: "application/pdf" },
        { name: "photo1.jpg", url: "/files/2", type: "image/jpeg" },
        { name: "photo2.png", url: "/files/3", type: "image/png" },
        { name: "text.txt", url: "/files/4", type: "text/plain" },
      ];

      const vm = await FileListViewModel.__create(tile, node, fileData);
      const images = await vm.getImages();

      assert.equal(images.length, 2);
      assert.isTrue(images.every(img => img.isImage()));
    });

    test("should find file by name via getByName", async () => {
      const { FileListViewModel } = await import("../src/main");
      const alizarin = await import("alizarin");

      const node = new alizarin.staticTypes.StaticNode({
        nodeid: "file-node-1",
        name: "Attachments",
        datatype: "file-list",
        nodegroup_id: "ng-1",
        alias: "attachments",
        graph_id: "test-graph",
        is_collector: true,
        isrequired: false,
        exportable: true,
        config: {},
      });

      const tile = new alizarin.staticTypes.StaticTile({
        tileid: "tile-1",
        nodegroup_id: "ng-1",
        resourceinstance_id: "resource-1",
        data: new Map(),
      });

      const fileData = [
        { name: "report.pdf", url: "/files/1" },
        { name: "photo.jpg", url: "/files/2" },
      ];

      const vm = await FileListViewModel.__create(tile, node, fileData);

      const found = await vm.getByName("report.pdf");
      assert.isDefined(found);
      assert.equal(found!.name, "report.pdf");

      const notFound = await vm.getByName("nonexistent.txt");
      assert.isNull(notFound);
    });

    test("should find file by ID via getById", async () => {
      const { FileListViewModel } = await import("../src/main");
      const alizarin = await import("alizarin");

      const node = new alizarin.staticTypes.StaticNode({
        nodeid: "file-node-1",
        name: "Attachments",
        datatype: "file-list",
        nodegroup_id: "ng-1",
        alias: "attachments",
        graph_id: "test-graph",
        is_collector: true,
        isrequired: false,
        exportable: true,
        config: {},
      });

      const tile = new alizarin.staticTypes.StaticTile({
        tileid: "tile-1",
        nodegroup_id: "ng-1",
        resourceinstance_id: "resource-1",
        data: new Map(),
      });

      const fileData = [
        { name: "file1.pdf", url: "/files/1", file_id: "uuid-1" },
        { name: "file2.pdf", url: "/files/2", file_id: "uuid-2" },
      ];

      const vm = await FileListViewModel.__create(tile, node, fileData);

      const found = await vm.getById("uuid-2");
      assert.isDefined(found);
      assert.equal(found!.file_id, "uuid-2");
      assert.equal(found!.name, "file2.pdf");

      const notFound = await vm.getById("nonexistent-uuid");
      assert.isNull(notFound);
    });

    test("should serialize via forJson", async () => {
      const { FileListViewModel } = await import("../src/main");
      const alizarin = await import("alizarin");

      const node = new alizarin.staticTypes.StaticNode({
        nodeid: "file-node-1",
        name: "Attachments",
        datatype: "file-list",
        nodegroup_id: "ng-1",
        alias: "attachments",
        graph_id: "test-graph",
        is_collector: true,
        isrequired: false,
        exportable: true,
        config: {},
      });

      const tile = new alizarin.staticTypes.StaticTile({
        tileid: "tile-1",
        nodegroup_id: "ng-1",
        resourceinstance_id: "resource-1",
        data: new Map(),
      });

      const fileData = [
        { name: "file1.pdf", url: "/files/1", file_id: "uuid-1", type: "application/pdf" },
      ];

      const vm = await FileListViewModel.__create(tile, node, fileData);
      const result = await vm.forJson();

      assert.isDefined(result);
      assert.isArray(result);
      assert.equal(result!.length, 1);
      assert.equal(result![0]!.name, "file1.pdf");
    });
  });

  // ===========================================================================
  // Integration Tests
  // ===========================================================================

  describe("Integration", () => {
    test("getViewModel should return FileListViewModel for file-list datatype", async () => {
      // Ensure extension is registered
      await import("../src/main");
      const alizarin = await import("alizarin");
      const { getViewModel } = alizarin.viewModels;

      const node = new alizarin.staticTypes.StaticNode({
        nodeid: "file-node-1",
        name: "Attachments",
        datatype: "file-list",
        nodegroup_id: "ng-1",
        alias: "attachments",
        graph_id: "test-graph",
        is_collector: true,
        isrequired: false,
        exportable: true,
        config: {},
      });

      const tile = new alizarin.staticTypes.StaticTile({
        tileid: "tile-1",
        nodegroup_id: "ng-1",
        resourceinstance_id: "resource-1",
        data: new Map(),
      });

      const fileData = [
        { name: "test.pdf", url: "/files/test", type: "application/pdf" },
      ];

      // Create a mock parent pseudo
      const mockParentPseudo = {
        parent: { $: null },
        describeField: () => null,
        describeFieldGroup: () => null,
      };

      const vm = await getViewModel(mockParentPseudo as any, tile, node, fileData);

      assert.isDefined(vm);
      // Should be a FileListViewModel (which is an Array)
      assert.isTrue(Array.isArray(vm));
      assert.equal((vm as any[]).length, 1);
    });
  });
});
