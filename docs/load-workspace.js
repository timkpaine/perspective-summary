import * as perspective from "../node_modules/@finos/perspective/dist/cdn/perspective.js";

const workspace = document.getElementById("workspace");

const DEFAULT_LAYOUT = {
  sizes: [0.15, 0.85],
  detail: {
    main: {
      type: "split-area",
      orientation: "vertical",
      children: [
        {
          type: "split-area",
          orientation: "horizontal",
          children: [
            {
              type: "tab-area",
              widgets: ["FOUR"],
              currentIndex: 0,
            },
          ],
          sizes: [1],
        },
        { type: "tab-area", widgets: ["One"], currentIndex: 0 },
      ],
      sizes: [0.1, 0.9],
    },
  },
  mode: "globalFilters",
  master: {
    widgets: ["ZERO", "ONE", "TWO", "THREE"],
    sizes: [0.2, 0.2, 0.2, 0.4],
  },
  viewers: {
    ZERO: {
      plugin: "Datagrid",
      settings: false,
      theme: "Pro Dark",
      title: "",
      group_by: ["Ship Mode"],
      columns: ["Ship Mode"],
      master: true,
      table: "superstore",
      linked: false,
      selectable: "",
    },
    ONE: {
      plugin: "Datagrid",
      settings: false,
      theme: "Pro Dark",
      title: "",
      group_by: ["Category"],
      columns: ["Category"],
      master: true,
      table: "superstore",
      linked: false,
      selectable: "",
    },
    TWO: {
      plugin: "Datagrid",
      settings: false,
      theme: "Pro Dark",
      title: "",
      group_by: ["Segment"],
      columns: ["Segment"],
      master: true,
      table: "superstore",
      linked: false,
      selectable: "",
    },
    THREE: {
      plugin: "Datagrid",
      settings: false,
      theme: "Pro Dark",
      title: "",
      group_by: ["Customer ID"],
      columns: ["Customer ID"],
      aggregates: { "Customer ID": "last" },
      master: true,
      table: "superstore",
      linked: false,
      selectable: "",
    },
    FOUR: {
      plugin: "Summary",
      plugin_config: {
        align: "horizontal",
        align_header: "right",
        theme: "modern",
      },
      settings: false,
      theme: "Pro Dark",
      title: "Summary",
      group_by: ["Customer ID"],
      columns: ["Quantity", "Discount", "Profit", "Sales", "Order Date"],
      aggregates: { "Order Date": "last" },
      master: false,
      table: "superstore",
      linked: false,
    },
    One: {
      plugin: "Datagrid",
      settings: false,
      theme: "Pro Dark",
      title: "Data",
      columns: [
        "Row ID",
        "Order ID",
        "Order Date",
        "Ship Date",
        "Ship Mode",
        "Customer ID",
        "Segment",
        "Country",
        "City",
        "State",
        "Postal Code",
        "Region",
        "Product ID",
        "Category",
        "Sub-Category",
        "Sales",
        "Quantity",
        "Discount",
        "Profit",
      ],
      master: false,
      table: "superstore",
      linked: false,
    },
  },
};

async function load() {
  let resp = await fetch("superstore.csv");
  let csv = await resp.text();
  const worker = perspective.shared_worker();
  workspace.tables.set("superstore", await worker.table(csv));
  workspace.restore(DEFAULT_LAYOUT);
}

await load();
