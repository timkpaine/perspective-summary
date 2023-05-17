/// <reference types="less" />
import { Plugin } from 'esbuild';
export interface LoaderOptions {
    filter?: RegExp;
}
/** Less-loader for esbuild */
export declare const lessLoader: (options?: Less.Options, loaderOptions?: LoaderOptions) => Plugin;
