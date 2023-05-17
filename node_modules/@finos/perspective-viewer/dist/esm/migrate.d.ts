/******************************************************************************
 *
 * Copyright (c) 2017, the Perspective Authors.
 *
 * This file is part of the Perspective library, distributed under the terms of
 * the Apache License 2.0.  The full license can be found in the LICENSE file.
 *
 */
/**
 * A migration utility for `@finos/perspective-viewer` and
 * `@finos/perspective-workspace` persisted state objects.  If you have an
 * application which persists tokens returned by the `.save()` method of a
 * Perspective Custom Element, and you want to upgrade Perspective to the latest
 * version, this module is for you!  You know who you are!
 *
 * Say you have a `<perspective-viewer>` Custom Element from
 * `@finos/perspective-viewer>=0.8.3`, and have persisted an arbitrary persistence
 * token object:
 *
 * ```javascript
 * const old_elem = document.querySelector("perspective-viewer");
 * const old_token = await old_elem.save();
 * ```
 *
 * To migrate this token to the version of `@finos/perspective-migrate` itself:
 *
 * ```javascript
 * import {convert} from "@finos/perspective-viewer`;
 *
 * // ...
 *
 * const new_elem = document.querySelector("perspective-viewer");
 * const new_token = convert(old_token);
 * await new_elem.restore(new_token);
 * ```
 *
 * `convert` can also be imported in node for converting persisted tokens
 * outside the browser.
 *
 * ```javascript
 * const {convert} = require("@finos/perspective-viewer/dist/cjs/migrate.js");
 * ```
 * @param old the layout to convert, in `<perspective-viewer>` or
 * `<perspective-workspace>` format.
 * @param options a `PerspectiveConvertOptions` object specifying the convert
 * options for this call.
 * @returns a layout for either `<perspective-viewer>` or
 * `<perspective-workspace>`, updated to the perspective version of this
 * script's package.
 */
export declare function convert(old: Record<string, unknown> | ArrayBuffer | string, { warn, replace_defaults }?: PerspectiveConvertOptions): Record<string, unknown> | ArrayBuffer | string;
type PerspectiveConvertOptions = {
    warn?: boolean;
    replace_defaults?: boolean;
};
export {};
