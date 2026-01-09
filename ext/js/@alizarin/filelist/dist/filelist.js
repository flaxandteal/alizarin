import { registerDisplaySerializer, utils, viewModels, wasmReady } from "alizarin";
var FileListItem = class {
	accepted;
	altText;
	attribution;
	content;
	description;
	file_id;
	index;
	lastModified;
	name;
	path;
	selected;
	size;
	status;
	title;
	type;
	url;
	renderer;
	constructor(e) {
		this.accepted = e.accepted ?? !1, this.altText = e.altText, this.attribution = e.attribution, this.content = e.content, this.description = e.description, this.file_id = e.file_id, this.index = e.index, this.lastModified = e.lastModified, this.name = e.name || "", this.path = e.path, this.selected = e.selected ?? !1, this.size = e.size, this.status = e.status, this.title = e.title, this.type = e.type, this.url = e.url, this.renderer = e.renderer;
	}
	toDisplayString(e) {
		let r = e || utils.getCurrentLanguage() || "en";
		if (this.title) {
			if (this.title[r]?.value) return this.title[r].value;
			for (let e of Object.values(this.title)) if (e.value) return e.value;
		}
		return this.name ? this.name : this.file_id || "(unnamed file)";
	}
	getAltText(e) {
		let r = e || utils.getCurrentLanguage() || "en";
		return this.altText && this.altText[r]?.value ? this.altText[r].value : null;
	}
	isImage() {
		return this.type?.startsWith("image/") ?? !1;
	}
	toJson() {
		let e = {
			name: this.name,
			accepted: this.accepted,
			selected: this.selected
		};
		return this.altText && (e.altText = this.altText), this.attribution && (e.attribution = this.attribution), this.content && (e.content = this.content), this.description && (e.description = this.description), this.file_id && (e.file_id = this.file_id), this.index !== void 0 && (e.index = this.index), this.lastModified && (e.lastModified = this.lastModified), this.path && (e.path = this.path), this.size !== void 0 && (e.size = this.size), this.status && (e.status = this.status), this.title && (e.title = this.title), this.type && (e.type = this.type), this.url && (e.url = this.url), this.renderer && (e.renderer = this.renderer), e;
	}
};
wasmReady.then(() => {
	registerDisplaySerializer("file-list", (e, n) => e ? Array.isArray(e) ? e.length === 0 ? null : e.map((e) => !e || typeof e != "object" ? null : new FileListItem(e).toDisplayString(n)).filter((e) => e !== null).join(", ") : typeof e == "object" && e ? new FileListItem(e).toDisplayString(n) : null : null);
});
var FileItemViewModel = class extends String {
	_ = void 0;
	__parentPseudo;
	describeField = () => this.__parentPseudo ? this.__parentPseudo.describeField() : null;
	describeFieldGroup = () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null;
	_file;
	constructor(e) {
		super(e.toDisplayString()), this._file = e;
	}
	async forJson() {
		return this._file.toJson();
	}
	getValue() {
		return this._file;
	}
	get name() {
		return this._file.name;
	}
	get url() {
		return this._file.url;
	}
	get file_id() {
		return this._file.file_id;
	}
	get fileType() {
		return this._file.type;
	}
	get size() {
		return this._file.size;
	}
	isImage() {
		return this._file.isImage();
	}
	getAltText(e) {
		return this._file.getAltText(e);
	}
	async __asTileData() {
		return this._file ? this._file.toJson() : null;
	}
}, FileListViewModel = class e extends Array {
	_ = void 0;
	__parentPseudo;
	describeField = () => this.__parentPseudo ? this.__parentPseudo.describeField() : null;
	describeFieldGroup = () => this.__parentPseudo ? this.__parentPseudo.describeFieldGroup() : null;
	_value = null;
	async forJson() {
		let e = await this._value;
		return e ? Promise.all(e.map(async (e) => e ? e.forJson() : null)) : null;
	}
	async getImages() {
		let e = Array.from(this);
		return (await Promise.all(e)).filter((e) => e !== null && e.isImage());
	}
	async getByName(e) {
		let n = Array.from(this);
		return (await Promise.all(n)).find((n) => n !== null && n.name === e) ?? null;
	}
	async getById(e) {
		let n = Array.from(this);
		return (await Promise.all(n)).find((n) => n !== null && n.file_id === e) ?? null;
	}
	static async __create(n, r, i, o = null) {
		let s = r.nodeid, c = [];
		if (n.data.has(s) || n.data.set(s, null), i != null) {
			if (n.data.set(s, []), i instanceof Promise) {
				let a = await i;
				return e.__create(n, r, a, o);
			}
			let l;
			if (Array.isArray(i)) l = i;
			else if (typeof i == "object") l = [i];
			else throw Error(`Cannot set file-list value on node ${s} except via array or object: ${JSON.stringify(i)}`);
			c = l.map((e, n) => {
				if (e instanceof FileItemViewModel) return e;
				if (typeof e == "object" && e) {
					let r = e;
					return r.index === void 0 && (r.index = n), new FileItemViewModel(new FileListItem(r));
				}
				return null;
			}), Promise.all(c).then((e) => {
				let r = e.filter((e) => e !== null).map((e) => e.getValue().toJson());
				n.data.set(s, r);
			});
		} else c = [];
		let l = new e(...c);
		return l._value = Promise.all(c), l;
	}
	async __asTileData() {
		return this.forJson();
	}
}, FileListDataType = class {
	static async __create(e, n, r, i) {
		return FileListViewModel.__create(e, n, r, i);
	}
};
viewModels.CUSTOM_DATATYPES.set("file-list", FileListDataType);
export { FileItemViewModel, FileListDataType, FileListItem, FileListViewModel };
