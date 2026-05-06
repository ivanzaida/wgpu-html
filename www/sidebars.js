/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  docs: [
    {
      type: 'doc',
      id: 'index',
      label: 'Welcome',
    },
    {
      type: 'category',
      label: 'Getting Started',
      collapsed: false,
      items: [
        'getting-started/overview',
        'getting-started/installation',
        'getting-started/quick-start',
      ],
    },
    {
      type: 'category',
      label: 'HTML Markup',
      items: [
        'html/overview',
        'html/element-index',
        'html/elements',
        'html/parsing',
        'html/dom-api',
      ],
    },
    {
      type: 'category',
      label: 'CSS & Styling',
      items: [
        'css/overview',
        'css/syntax',
        'css/property-index',
        'css/selectors',
        'css/cascade',
        'css/box-model',
        'css/colors-backgrounds',
        'css/typography',
        'css/flexbox',
        'css/grid',
        'css/positioned',
        'css/overflow',
        'css/variables',
        'css/math-functions',
        'css/css-roadmap',
      ],
    },
    {
      type: 'category',
      label: 'Layout Engine',
      items: [
        'layout/overview',
        'layout/block',
        'layout/flexbox',
        'layout/grid',
        'layout/positioned',
        'layout/inline',
      ],
    },
    {
      type: 'category',
      label: 'Text & Typography',
      items: [
        'text/overview',
        'text/fonts',
        'text/shaping',
      ],
    },
    {
      type: 'category',
      label: 'Rendering',
      items: [
        'rendering/overview',
        'rendering/pipelines',
        'rendering/painting',
      ],
    },
    {
      type: 'category',
      label: 'Interactivity',
      items: [
        'interactivity/overview',
        'interactivity/events',
        'interactivity/focus-keyboard',
        'interactivity/forms',
        'interactivity/text-selection',
        'interactivity/scrolling',
      ],
    },
    {
      type: 'category',
      label: 'Rust Integration',
      items: [
        'rust-integration/overview',
        'rust-integration/integrating',
        'rust-integration/winit-harness',
        'rust-integration/bevy-integration',
        'rust-integration/egui-backend',
        'rust-integration/screenshots',
      ],
    },
    {
      type: 'category',
      label: 'Component Framework',
      items: [
        'component-framework/index',
        'component-framework/component-trait',
        'component-framework/el-dsl',
        'component-framework/ctx',
        'component-framework/store',
        'component-framework/rendering',
      ],
    },
    {
      type: 'doc',
      id: 'devtools',
      label: 'Devtools',
    },
    {
      type: 'doc',
      id: 'comparison-wgpu-html-vs-rmlui',
      label: 'vs RmlUI Comparison',
    },
    {
      type: 'doc',
      id: 'status',
      label: 'Implementation Status',
    },
  ],
};

export default sidebars;
