import GLOBAL_STYLE from "../css/global.css";
import DEFAULT from "../../dist/css/perspective-viewer-summary.css";
import MINIMAL from "../../dist/css/perspective-viewer-summary-minimal.css";
import MODERN from "../../dist/css/perspective-viewer-summary-modern.css";
import dayjs from "dayjs";

const _ALIGN_OPTIONS = ["horizontal", "vertical"];
const _ALIGN_DEFAULT = "horizontal";
const _ALIGN_HEADER_OPTIONS = ["top", "bottom", "left", "right"];
const _ALIGN_HEADER_DEFAULTS = {
  default: {
    horizontal: "top",
    vertical: "top",
  },
  minimal: {
    horizontal: "top",
    vertical: "top",
  },
  modern: {
    horizontal: "right",
    vertical: "right",
  },
};

const THEMES = {
  default: DEFAULT,
  minimal: MINIMAL,
  modern: MODERN,
};

export class PerspectiveViewerSummaryPluginElement extends HTMLElement {
  constructor() {
    super();
    /*
     * Config:
     *  align: str = "vertical" | "horizontal",  // align items vertically or horizontally, default is horizontal
     *  align_header: str = "top" | "bottom" | "left" | "right"
     *  format: {[col: str]: int|str }, // round numbers, truncate strings, format dates
     *  header_class: str, // css class to add to all headers
     *  data_class: str, // css class to add to all datas
     *  header_classes: {[col: str]: str}, // css class to add to specific headers
     *  data_classes: {[col: str]: str}, //  css class to add to specific datas
     *  theme: str = "default" | "minimal" | "modern"
     */
    this._config = {
      plugin_config: {
        theme: "default",
        align: _ALIGN_DEFAULT,
        align_header: undefined,
        format: {},
        header_class: "",
        data_class: "",
        header_classes: {},
        data_classes: {},
      },
    };

    // store data and dom elements
    this._data = null;
    this._schema = null;
    this._container = null;
    this._style = null;
    this._loaded = false;
  }

  connectedCallback() {
    if (!this._loaded) {
      this._shadow = this.attachShadow({ mode: "open" });

      this._container = document.createElement("div");
      this._container.classList.add("summary-container");

      this._style = document.createElement("style");

      this._shadow.appendChild(this._style);
      this._shadow.appendChild(this._container);

      this._global_style = document.createElement("style");
      this._global_style.textContent = GLOBAL_STYLE;
      document.head.appendChild(this._global_style);
    }
    this._loaded = true;
  }

  disconnectedCallback() {}

  async activate(view) {
    // nothing to do
  }

  get name() {
    // Plugin name is "Summary"
    return "Summary";
  }

  get select_mode() {
    // select any number of cols
    return "select";
  }

  get min_config_columns() {
    // at least 1 column
    return 1;
  }

  async draw(view) {
    await this.render(view);
  }
  async update(view) {
    await this.render(view);
  }

  async render(view) {
    // pull config
    const config = await view.get_config();

    this._config = {
      ...this._config,
      ...config,
      plugin_config: this._config.plugin_config,
    };

    this._schema = await view.schema();

    // get the columns being displayed
    const columns = config.columns;

    // access the first row of data, the total aggregation
    const data_window = {
      start_row: 0,
      start_col: 0,
      end_row: 1,
      end_col: columns.length,
      id: false,
    };

    // get the data itself
    this._data = await view.to_columns(data_window);

    // set class based on alignment
    this.format(config.plugin_config);
  }

  async resize() {
    // TODO nothing yet
  }

  async clear() {
    // TODO
  }

  save() {
    return { ...this._config.plugin_config };
  }

  restore(token) {
    this.format(token);
  }

  validate(restore) {
    restore = restore || this._config.plugin_config;

    // save if theme is changing
    const theme_change =
      restore.theme !== undefined &&
      restore.theme !== this._config.plugin_config.theme;

    // set theme
    this._config.plugin_config.theme =
      restore.theme || this._config.plugin_config.theme;

    // ensure theme is valid
    if (Object.keys(THEMES).indexOf(this._config.plugin_config.theme) < 0) {
      this._config.plugin_config.theme = "default";
    }

    // set alignment
    this._config.plugin_config.align =
      restore.align || this._config.plugin_config.align;

    // set alignment to default if invalid
    if (_ALIGN_OPTIONS.indexOf(this._config.plugin_config.align) < 0) {
      this._config.plugin_config.align = _ALIGN_DEFAULT;
    }

    // set header alignment
    this._config.plugin_config.align_header =
      restore.align_header || this._config.plugin_config.align_header;

    // set header alignment to default if invalid
    if (
      _ALIGN_HEADER_OPTIONS.indexOf(this._config.plugin_config.align_header) < 0
    ) {
      this._config.plugin_config.align_header =
        _ALIGN_HEADER_DEFAULTS[this._config.plugin_config.theme][
          this._config.plugin_config.align
        ];
    }

    // set default header alignment if not set
    if (
      this._config.plugin_config.align_header === undefined ||
      (theme_change && restore.align_header === undefined)
    ) {
      this._config.plugin_config.align_header =
        _ALIGN_HEADER_DEFAULTS[this._config.plugin_config.theme][
          this._config.plugin_config.align
        ];
    }

    // handle other restores
    this._config.plugin_config.header_class =
      restore.header_class || this._config.plugin_config.header_class;
    this._config.plugin_config.data_class =
      restore.data_class || this._config.plugin_config.data_class;
    this._config.plugin_config.header_classes =
      restore.header_classes || this._config.plugin_config.header_classes;
    this._config.plugin_config.data_classes =
      restore.data_classes || this._config.plugin_config.data_classes;

    // ensure format is present
    this._config.plugin_config.format =
      restore.format || this._config.plugin_config.format || {};
  }

  format(restore) {
    if (!this._loaded) {
      return;
    }

    // validate config
    this.validate(restore);

    // setup style
    this._style.textContent =
      THEMES[this._config.plugin_config.theme] || DEFAULT;

    // get the columns being displayed
    const columns = this._config.columns;

    // get the aggregations for those columns
    const aggregations = this._config.aggregates;

    // if we're not pivoted, we don't display data
    const is_pivoted =
      this._data.__ROW_PATH__ && this._data.__ROW_PATH__.length > 0;

    // keep in a map so we can format things individuallyt
    const _entries = new Map();

    columns.forEach((col) => {
      // container for data
      const col_container = document.createElement("div");
      col_container.classList.add("summary-column");

      // header
      const header_data = document.createElement("span");
      header_data.classList.add("summary-header");

      const header_divider = document.createElement("span");
      header_divider.classList.add("summary-header-divider");

      // add user provided classes
      if (this._config.plugin_config.header_class) {
        header_data.classList.add(this._config.plugin_config.header_class);
      }
      if (
        this._config.plugin_config.header_classes &&
        this._config.plugin_config.header_classes[col]
      ) {
        header_data.classList.add(
          this._config.plugin_config.header_classes[col]
        );
      }

      // put column name in content
      header_data.textContent = col;

      // data
      const data_data = document.createElement("span");
      data_data.classList.add("summary-data");

      // add user provided classes
      if (this._config.plugin_config.data_class) {
        data_data.classList.add(this._config.plugin_config.data_class);
      }
      if (
        this._config.plugin_config.data_classes &&
        this._config.plugin_config.data_classes[col]
      ) {
        data_data.classList.add(this._config.plugin_config.data_classes[col]);
      }

      // add to container
      if (
        ["top", "left"].indexOf(this._config.plugin_config.align_header) >= 0
      ) {
        col_container.appendChild(header_data);
        col_container.appendChild(header_divider);
        col_container.appendChild(data_data);
      } else {
        col_container.appendChild(data_data);
        col_container.appendChild(header_divider);
        col_container.appendChild(header_data);
      }

      // put in map
      _entries.set(col, col_container);

      if (is_pivoted) {
        // pull the data from the first row
        let datum = this._data[col];

        // show the aggregation type in tooltip, using type-specific variants
        // NOTE: don't pull from schema as this will reflect the aggregate's type
        const aggregate =
          aggregations[col] || (Number.isNaN(datum) ? "count" : "sum");
        data_data.title = `${aggregate}("${col}")`;

        // grab formatter
        const formatter = this._config.plugin_config.format[col];

        // format the data if necessary
        if (["integer"].indexOf(this._schema[col]) >= 0) {
          // round to `n` decimals
          if (this._config.plugin_config.theme === "modern") {
            // negatives not supported everywhere
            datum = `${+datum < 0 ? "-" : ""}${new Intl.NumberFormat("en-US", {
              maximumFractionDigits: +(formatter || 0),
              notation: "compact",
              compactDisplay: "short",
            }).format(Math.abs(+datum))}`;
          } else {
            datum = Number(datum).toFixed(+(formatter || 0));
          }
        } else if (["float"].indexOf(this._schema[col]) >= 0) {
          if (this._config.plugin_config.theme === "modern") {
            datum = new Intl.NumberFormat("en-US", {
              maximumFractionDigits: +(formatter || 1),
              notation: "compact",
              compactDisplay: "short",
            }).format(Number(datum));
          } else {
            datum = Number(datum).toFixed(+(formatter || 1));
          }
        } else if (["boolean"].indexOf(this._schema[col]) >= 0) {
          // do nothing
        } else if (["datetime", "date"].indexOf(this._schema[col]) >= 0) {
          // format
          if (this._config.plugin_config.theme === "modern") {
            datum = dayjs(+datum).format(formatter || "MMM D");
          } else {
            datum = dayjs(+datum).format(formatter);
          }
        } else {
          // truncate the string to `n` digits
          if (formatter !== undefined) {
            datum = new String(datum).substring(0, +formatter);
          }
        }

        // the data itself
        data_data.textContent = datum;

        // add classes to data if we have it and we have data
        if (
          this._config.plugin_config.data_classes &&
          this._config.plugin_config.data_classes[col]
        ) {
          data_data.classList.add(this._config.plugin_config.data_classes[col]);
        }
      } else {
        data_data.textContent = "--";
      }

      // set class based on alignment
      this._container.classList.remove("align-horizontal");
      this._container.classList.remove("align-vertical");
      this._container.classList.add(
        `align-${this._config.plugin_config.align}`
      );

      // set class based on header alignment
      this._container.classList.remove("align-header-top");
      this._container.classList.remove("align-header-bottom");
      this._container.classList.remove("align-header-left");
      this._container.classList.remove("align-header-right");
      this._container.classList.add(
        `align-header-${this._config.plugin_config.align_header}`
      );
    });

    // clear it out
    while (this._container.lastChild) {
      this._container.removeChild(this._container.lastChild);
    }

    // and refill
    _entries.forEach((value, key) => {
      this._container.appendChild(value);
    });
  }

  async restyle(view) {
    // TODO
  }

  delete() {
    // TODO
  }
}

customElements.define(
  "perspective-viewer-summary",
  PerspectiveViewerSummaryPluginElement
);

function _register_global_styles() {
  const style = document.createElement("style");
  style.textContent = STYLE;
  document.head.insertBefore(style, document.head.firstChild);
}

function register_element() {
  customElements
    .get("perspective-viewer")
    .registerPlugin("perspective-viewer-summary");
}

customElements.whenDefined("perspective-viewer").then(register_element);
