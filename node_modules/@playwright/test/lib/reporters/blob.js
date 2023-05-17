"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.BlobReporter = void 0;
exports.createMergedReport = createMergedReport;
var _fs = _interopRequireDefault(require("fs"));
var _path = _interopRequireDefault(require("path"));
var _utils = require("playwright-core/lib/utils");
var _zipBundle = require("playwright-core/lib/zipBundle");
var _stream = require("stream");
var _teleReceiver = require("../isomorphic/teleReceiver");
var _dot = _interopRequireDefault(require("../reporters/dot"));
var _empty = _interopRequireDefault(require("../reporters/empty"));
var _github = _interopRequireDefault(require("../reporters/github"));
var _json = _interopRequireDefault(require("../reporters/json"));
var _junit = _interopRequireDefault(require("../reporters/junit"));
var _line = _interopRequireDefault(require("../reporters/line"));
var _list = _interopRequireDefault(require("../reporters/list"));
var _loadUtils = require("../runner/loadUtils");
var _html = _interopRequireWildcard(require("./html"));
var _teleEmitter = require("./teleEmitter");
function _getRequireWildcardCache(nodeInterop) { if (typeof WeakMap !== "function") return null; var cacheBabelInterop = new WeakMap(); var cacheNodeInterop = new WeakMap(); return (_getRequireWildcardCache = function (nodeInterop) { return nodeInterop ? cacheNodeInterop : cacheBabelInterop; })(nodeInterop); }
function _interopRequireWildcard(obj, nodeInterop) { if (!nodeInterop && obj && obj.__esModule) { return obj; } if (obj === null || typeof obj !== "object" && typeof obj !== "function") { return { default: obj }; } var cache = _getRequireWildcardCache(nodeInterop); if (cache && cache.has(obj)) { return cache.get(obj); } var newObj = {}; var hasPropertyDescriptor = Object.defineProperty && Object.getOwnPropertyDescriptor; for (var key in obj) { if (key !== "default" && Object.prototype.hasOwnProperty.call(obj, key)) { var desc = hasPropertyDescriptor ? Object.getOwnPropertyDescriptor(obj, key) : null; if (desc && (desc.get || desc.set)) { Object.defineProperty(newObj, key, desc); } else { newObj[key] = obj[key]; } } } newObj.default = obj; if (cache) { cache.set(obj, newObj); } return newObj; }
function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }
/**
 * Copyright (c) Microsoft Corporation.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

class BlobReporter extends _teleEmitter.TeleReporterEmitter {
  constructor(options) {
    super(message => this._messages.push(message));
    this._messages = [];
    this._options = void 0;
    this._outputFile = void 0;
    this._options = options;
  }
  onBegin(config, suite) {
    super.onBegin(config, suite);
    this._computeOutputFileName(config);
  }
  async onEnd(result) {
    await super.onEnd(result);
    _fs.default.mkdirSync(_path.default.dirname(this._outputFile), {
      recursive: true
    });
    const lines = this._messages.map(m => JSON.stringify(m) + '\n');
    await zipReport(this._outputFile, lines);
  }
  _computeOutputFileName(config) {
    const outputDir = this._resolveOutputDir();
    let shardSuffix = '';
    if (config.shard) {
      const paddedNumber = `${config.shard.current}`.padStart(`${config.shard.total}`.length, '0');
      shardSuffix = `-${paddedNumber}-of-${config.shard.total}`;
    }
    this._outputFile = _path.default.join(outputDir, `report${shardSuffix}.zip`);
  }
  _resolveOutputDir() {
    const {
      outputDir
    } = this._options;
    if (outputDir) return _path.default.resolve(this._options.configDir, outputDir);
    return (0, _html.defaultReportFolder)(this._options.configDir);
  }
}
exports.BlobReporter = BlobReporter;
async function createMergedReport(config, dir, reporterName) {
  var _reporterName, _config$config$report;
  const shardFiles = await sortedShardFiles(dir);
  const events = await mergeEvents(dir, shardFiles);
  const defaultReporters = {
    dot: _dot.default,
    line: _line.default,
    list: _list.default,
    github: _github.default,
    json: _json.default,
    junit: _junit.default,
    null: _empty.default,
    html: _html.default,
    blob: BlobReporter
  };
  (_reporterName = reporterName) !== null && _reporterName !== void 0 ? _reporterName : reporterName = 'list';
  const arg = (_config$config$report = config.config.reporter.find(([reporter, arg]) => reporter === reporterName)) === null || _config$config$report === void 0 ? void 0 : _config$config$report[1];
  const options = {
    ...arg,
    configDir: process.cwd(),
    outputFolder: dir
  };
  let reporter;
  if (reporterName in defaultReporters) {
    reporter = new defaultReporters[reporterName](options);
  } else {
    const reporterConstructor = await (0, _loadUtils.loadReporter)(config, reporterName);
    reporter = new reporterConstructor(options);
  }
  const receiver = new _teleReceiver.TeleReporterReceiver(_path.default.sep, reporter);
  for (const event of events) await receiver.dispatch(event);
  console.log(`Done.`);
}
async function mergeEvents(dir, shardFiles) {
  const events = [];
  const beginEvents = [];
  const endEvents = [];
  for (const file of shardFiles) {
    const zipFile = new _utils.ZipFile(_path.default.join(dir, file));
    const entryNames = await zipFile.entries();
    const reportEntryName = entryNames.find(e => e.endsWith('.jsonl'));
    if (!reportEntryName) throw new Error(`Zip file ${file} does not contain a .jsonl file`);
    const reportJson = await zipFile.read(reportEntryName);
    const parsedEvents = reportJson.toString().split('\n').filter(line => line.length).map(line => JSON.parse(line));
    for (const event of parsedEvents) {
      // TODO: show remaining events?
      if (event.method === 'onError') throw new Error('Error in shard: ' + file);
      if (event.method === 'onBegin') beginEvents.push(event);else if (event.method === 'onEnd') endEvents.push(event);else events.push(event);
    }
  }
  return [mergeBeginEvents(beginEvents), ...events, mergeEndEvents(endEvents)];
}
function mergeBeginEvents(beginEvents) {
  if (!beginEvents.length) throw new Error('No begin events found');
  const projects = [];
  let totalWorkers = 0;
  for (const event of beginEvents) {
    totalWorkers += event.params.config.workers;
    const shardProjects = event.params.projects;
    for (const shardProject of shardProjects) {
      const mergedProject = projects.find(p => p.id === shardProject.id);
      if (!mergedProject) projects.push(shardProject);else mergeJsonSuites(shardProject.suites, mergedProject);
    }
  }
  const config = {
    ...beginEvents[0].params.config,
    workers: totalWorkers,
    shard: undefined
  };
  return {
    method: 'onBegin',
    params: {
      config,
      projects
    }
  };
}
function mergeJsonSuites(jsonSuites, parent) {
  for (const jsonSuite of jsonSuites) {
    const existingSuite = parent.suites.find(s => s.title === jsonSuite.title);
    if (!existingSuite) {
      parent.suites.push(jsonSuite);
    } else {
      mergeJsonSuites(jsonSuite.suites, existingSuite);
      existingSuite.tests.push(...jsonSuite.tests);
    }
  }
}
function mergeEndEvents(endEvents) {
  const result = {
    status: 'passed'
  };
  for (const event of endEvents) {
    const shardResult = event.params.result;
    if (shardResult.status === 'failed') result.status = 'failed';else if (shardResult.status === 'timedout' && result.status !== 'failed') result.status = 'timedout';else if (shardResult.status === 'interrupted' && result.status !== 'failed' && result.status !== 'timedout') result.status = 'interrupted';
  }
  return {
    method: 'onEnd',
    params: {
      result
    }
  };
}
async function sortedShardFiles(dir) {
  const files = await _fs.default.promises.readdir(dir);
  return files.filter(file => file.endsWith('.zip')).sort();
}
async function zipReport(zipFileName, lines) {
  const zipFile = new _zipBundle.yazl.ZipFile();
  const result = new _utils.ManualPromise();
  zipFile.on('error', error => result.reject(error));
  // TODO: feed events on the fly.
  const content = _stream.Readable.from(lines);
  zipFile.addReadStream(content, 'report.jsonl');
  zipFile.end();
  zipFile.outputStream.pipe(_fs.default.createWriteStream(zipFileName)).on('close', () => {
    result.resolve(undefined);
  });
  await result;
}