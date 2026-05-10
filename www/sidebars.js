/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  docs: [
    'intro',
    {
      type: 'category',
      label: 'Getting Started',
      collapsed: false,
      items: [
        'getting-started/overview',
        'getting-started/installation',
        'getting-started/quick-start',
        'getting-started/running-examples',
        'getting-started/project-structure',
      ],
    },
    {
      type: 'category',
      label: 'Engine',
      items: [
        'engine/architecture',
        'engine/rendering-pipeline',
        'engine/dom-model',
        'engine/css-engine',
        'engine/layout-engine',
        'engine/painting',
        'engine/text-rendering',
        'engine/events',
        'engine/input-handling',
        'engine/resource-loading',
      ],
    },
    {
      type: 'category',
      label: 'Features',
      items: [
        'features/supported-html',
        'features/supported-css',
        'features/supported-selectors',
        'features/supported-events',
        'features/forms-and-inputs',
        'features/fonts',
        'features/images',
        'features/scrolling',
        'features/clipping-and-overflow',
        'features/z-index-and-stacking',
      ],
    },
    {
      type: 'category',
      label: 'Integration',
      items: [
        'integration/embedding',
        'integration/wgpu-backend',
        'integration/windowing',
        'integration/egui-integration',
        'integration/custom-elements',
        'integration/native-apps',
      ],
    },
    {
      type: 'category',
      label: 'Examples',
      items: [
        'examples/simple-page',
        'examples/styled-page',
        'examples/forms',
        'examples/interactive-ui',
        'examples/custom-render-target',
        'examples/devtools-overlay',
      ],
    },
    {
      type: 'category',
      label: 'Reference',
      items: [
        'reference/public-api',
        'reference/configuration',
        'reference/feature-flags',
        'reference/error-types',
        'reference/limits',
        'reference/compatibility',
      ],
    },
    {
      type: 'category',
      label: 'Development',
      items: [
        'development/building-from-source',
        'development/running-tests',
        'development/debugging',
        'development/profiling',
        'development/adding-html-elements',
        'development/adding-css-properties',
        'development/adding-layout-features',
        'development/contributing',
      ],
    },
    {
      type: 'category',
      label: 'Roadmap',
      items: [
        'roadmap/roadmap',
        'roadmap/known-issues',
        'roadmap/browser-compatibility-goals',
      ],
    },
  ],
};

export default sidebars;