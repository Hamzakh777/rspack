import { Compiler } from "../Compiler";
import { ContainerPlugin, Exposes } from "./ContainerPlugin";
import { LibraryOptions, EntryRuntime } from "../config";

export interface ModuleFederationPluginOptions {
	exposes?: Exposes;
	filename?: string;
	library?: LibraryOptions;
	name: string;
	// remoteType?: ExternalsType;
	// remotes?: Remotes;
	runtime?: EntryRuntime;
	shareScope?: string;
	// shared?: Shared;
}

export class ModuleFederationPlugin {
	constructor(private _options: ModuleFederationPluginOptions) {}

	apply(compiler: Compiler) {
		const { _options: options } = this;
		const library = options.library || { type: "var", name: options.name };
		// const remoteType =
		// 	options.remoteType ||
		// 	(options.library && isValidExternalsType(options.library.type)
		// 		? /** @type {ExternalsType} */ (options.library.type)
		// 		: "script");
		if (
			library &&
			!compiler.options.output.enabledLibraryTypes!.includes(library.type)
		) {
			compiler.options.output.enabledLibraryTypes!.push(library.type);
		}
		compiler.hooks.afterPlugins.tap("ModuleFederationPlugin", () => {
			if (
				options.exposes &&
				(Array.isArray(options.exposes)
					? options.exposes.length > 0
					: Object.keys(options.exposes).length > 0)
			) {
				new ContainerPlugin({
					name: options.name,
					library,
					filename: options.filename,
					runtime: options.runtime,
					shareScope: options.shareScope,
					exposes: options.exposes
				}).apply(compiler);
			}
			// if (
			// 	options.remotes &&
			// 	(Array.isArray(options.remotes)
			// 		? options.remotes.length > 0
			// 		: Object.keys(options.remotes).length > 0)
			// ) {
			// 	new ContainerReferencePlugin({
			// 		remoteType,
			// 		shareScope: options.shareScope,
			// 		remotes: options.remotes
			// 	}).apply(compiler);
			// }
			// if (options.shared) {
			// 	new SharePlugin({
			// 		shared: options.shared,
			// 		shareScope: options.shareScope
			// 	}).apply(compiler);
			// }
		});
	}
}
