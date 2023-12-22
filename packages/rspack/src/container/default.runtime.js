// @ts-nocheck
var __module_federation_runtime__,
	__module_federation_runtime_plugins__,
	__module_federation_remote_infos__;
module.exports = function () {
	if (
		__webpack_require__.initializeSharingData ||
		__webpack_require__.initializeExposesData
	) {
		const override = (obj, key, value) => {
			if (!obj) return;
			if (obj[key]) obj[key] = value;
		};
		const merge = (obj, key, fn) => {
			const value = fn();
			if (Array.isArray(value)) {
				obj[key] ??= [];
				obj[key].push(...value);
			} else if (typeof value === "object" && value !== null) {
				obj[key] ??= {};
				Object.assign(obj[key], value);
			}
		};
		const early = (obj, key, initial) => {
			obj[key] ??= initial();
		};

		const remotesLoadingChunkMapping =
			__webpack_require__.remotesLoadingData?.chunkMapping ?? {};
		const remotesLoadingModuleIdToRemoteDataMapping =
			__webpack_require__.remotesLoadingData?.moduleIdToRemoteDataMapping ?? {};
		const initializeSharingScopeToInitDataMapping =
			__webpack_require__.initializeSharingData?.scopeToSharingDataMapping ??
			{};
		const consumesLoadingChunkMapping =
			__webpack_require__.consumesLoadingData?.chunkMapping ?? {};
		const consumesLoadingModuleToConsumeDataMapping =
			__webpack_require__.consumesLoadingData?.moduleIdToConsumeDataMapping ??
			{};
		const consumesLoadinginstalledModules = {};
		const initializeSharingInitPromises = [];
		const initializeSharingInitTokens = [];
		const containerShareScope =
			__webpack_require__.initializeExposesData?.containerShareScope;

		early(
			__webpack_require__,
			"federation",
			() => __module_federation_runtime__
		);

		early(
			__webpack_require__.federation,
			"consumesLoadingModuleToHandlerMapping",
			() => {
				const consumesLoadingModuleToHandlerMapping = {};
				for (let [moduleId, data] of Object.entries(
					consumesLoadingModuleToConsumeDataMapping
				)) {
					consumesLoadingModuleToHandlerMapping[moduleId] = {
						getter: data.fallback,
						shareInfo: {
							shareConfig: {
								fixedDependencies: false,
								requiredVersion: data.requiredVersion,
								strictVersion: data.strictVersion,
								singleton: data.singleton,
								eager: data.eager
							},
							scope: [data.shareScope]
						},
						shareKey: data.shareKey
					};
				}
				return consumesLoadingModuleToHandlerMapping;
			}
		);

		early(__webpack_require__.federation, "initOptions", () => ({}));
		early(
			__webpack_require__.federation.initOptions,
			"name",
			() => __webpack_require__.initializeSharingData?.uniqueName
		);
		early(__webpack_require__.federation.initOptions, "shared", () => {
			const shared = {};
			for (let [scope, stages] of Object.entries(
				initializeSharingScopeToInitDataMapping
			)) {
				for (let stage of stages) {
					if (Array.isArray(stage)) {
						const [name, version, factory, eager] = stage;
						if (shared[name]) {
							shared[name].scope.push(scope);
						} else {
							shared[name] = { version, get: factory, scope: [scope] };
						}
					}
				}
			}
			return shared;
		});
		merge(__webpack_require__.federation.initOptions, "remotes", () =>
			Object.values(__module_federation_remote_infos__).filter(
				remote => remote.externalType === "script"
			)
		);
		merge(
			__webpack_require__.federation.initOptions,
			"plugins",
			() => __module_federation_runtime_plugins__
		);

		early(__webpack_require__.federation, "bundlerRuntimeOptions", () => ({}));
		early(
			__webpack_require__.federation.bundlerRuntimeOptions,
			"remotes",
			() => ({})
		);
		early(
			__webpack_require__.federation.bundlerRuntimeOptions.remotes,
			"chunkMapping",
			() => remotesLoadingChunkMapping
		);
		early(
			__webpack_require__.federation.bundlerRuntimeOptions.remotes,
			"idToExternalAndNameMapping",
			() => remotesLoadingModuleIdToRemoteDataMapping
		);
		early(
			__webpack_require__.federation.bundlerRuntimeOptions.remotes,
			"webpackRequire",
			() => __webpack_require__
		);
		merge(
			__webpack_require__.federation.bundlerRuntimeOptions.remotes,
			"idToRemoteMap",
			() => {
				const idToRemoteMap = {};
				for (let [id, remoteData] of Object.entries(
					remotesLoadingModuleIdToRemoteDataMapping
				)) {
					const info = __module_federation_remote_infos__[remoteData[3]];
					if (info) idToRemoteMap[id] = [info];
				}
				return idToRemoteMap;
			}
		);

		override(
			__webpack_require__,
			"S",
			() => __module_federation_runtime__.bundlerRuntime.S
		);
		override(__webpack_require__.f, "remotes", (chunkId, promises) =>
			__module_federation_runtime__.bundlerRuntime.remotes({
				chunkId,
				promises,
				chunkMapping: remotesLoadingChunkMapping,
				idToExternalAndNameMapping: remotesLoadingModuleIdToRemoteDataMapping,
				idToRemoteMap:
					__webpack_require__.federation.bundlerRuntimeOptions.remotes
						.idToRemoteMap,
				webpackRequire: __webpack_require__
			})
		);
		override(__webpack_require__.f, "consumes", (chunkId, promises) =>
			__module_federation_runtime__.bundlerRuntime.consumes({
				chunkId,
				promises,
				chunkMapping: consumesLoadingChunkMapping,
				moduleToHandlerMapping:
					__webpack_require__.federation.consumesLoadingModuleToHandlerMapping,
				installedModules: consumesLoadinginstalledModules,
				webpackRequire: __webpack_require__
			})
		);
		override(__webpack_require__, "I", (name, initScope) =>
			__module_federation_runtime__.bundlerRuntime.I({
				shareScopeName: name,
				initScope,
				initPromises: initializeSharingInitPromises,
				initTokens: initializeSharingInitTokens,
				webpackRequire: __webpack_require__
			})
		);
		override(__webpack_require__, "initContainer", (shareScope, initScope) =>
			__module_federation_runtime__.bundlerRuntime.initContainerEntry({
				shareScope,
				initScope,
				shareScopeKey: containerShareScope,
				webpackRequire: __webpack_require__
			})
		);
		override(__webpack_require__, "getContainer", (module, getScope) => {
			var moduleMap = __webpack_require__.initializeExposesData.moduleMap;
			__webpack_require__.R = getScope;
			getScope = Object.prototype.hasOwnProperty.call(moduleMap, module)
				? moduleMap[module]()
				: Promise.resolve().then(() => {
						throw new Error(
							'Module "' + module + '" does not exist in container.'
						);
				  });
			__webpack_require__.R = undefined;
			return getScope;
		});

		__webpack_require__.federation.instance =
			__webpack_require__.federation.runtime.init(
				__webpack_require__.federation.initOptions
			);

		if (__webpack_require__.consumesLoadingData?.initialConsumes) {
			__webpack_require__.federation.bundlerRuntime.installInitialConsumes({
				webpackRequire: __webpack_require__,
				installedModules: consumesLoadinginstalledModules,
				initialConsumes:
					__webpack_require__.consumesLoadingData.initialConsumes,
				moduleToHandlerMapping:
					__webpack_require__.federation.consumesLoadingModuleToHandlerMapping
			});
		}
	}
};
