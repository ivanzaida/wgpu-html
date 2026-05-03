import React from 'react';
import ComponentCreator from '@docusaurus/ComponentCreator';

export default [
  {
    path: '/blog',
    component: ComponentCreator('/blog', '98b'),
    exact: true
  },
  {
    path: '/docs',
    component: ComponentCreator('/docs', '532'),
    routes: [
      {
        path: '/docs',
        component: ComponentCreator('/docs', '454'),
        routes: [
          {
            path: '/docs',
            component: ComponentCreator('/docs', '2ad'),
            routes: [
              {
                path: '/docs/',
                component: ComponentCreator('/docs/', '35b'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/comparison-wgpu-html-vs-rmlui',
                component: ComponentCreator('/docs/comparison-wgpu-html-vs-rmlui', '8b1'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/component-framework/',
                component: ComponentCreator('/docs/component-framework/', 'f0d'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/component-framework/component-trait',
                component: ComponentCreator('/docs/component-framework/component-trait', 'a36'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/component-framework/ctx',
                component: ComponentCreator('/docs/component-framework/ctx', 'bf5'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/component-framework/el-dsl',
                component: ComponentCreator('/docs/component-framework/el-dsl', '225'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/component-framework/rendering',
                component: ComponentCreator('/docs/component-framework/rendering', '43d'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/component-framework/store',
                component: ComponentCreator('/docs/component-framework/store', 'c5d'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/css/box-model',
                component: ComponentCreator('/docs/css/box-model', '834'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/css/cascade',
                component: ComponentCreator('/docs/css/cascade', '57e'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/css/colors-backgrounds',
                component: ComponentCreator('/docs/css/colors-backgrounds', '4d5'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/css/flexbox',
                component: ComponentCreator('/docs/css/flexbox', 'd6a'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/css/grid',
                component: ComponentCreator('/docs/css/grid', '5da'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/css/math-functions',
                component: ComponentCreator('/docs/css/math-functions', '04a'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/css/overflow',
                component: ComponentCreator('/docs/css/overflow', '8c7'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/css/overview',
                component: ComponentCreator('/docs/css/overview', '9e8'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/css/positioned',
                component: ComponentCreator('/docs/css/positioned', '0cb'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/css/property-index',
                component: ComponentCreator('/docs/css/property-index', '180'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/css/selectors',
                component: ComponentCreator('/docs/css/selectors', '327'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/css/syntax',
                component: ComponentCreator('/docs/css/syntax', '465'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/css/typography',
                component: ComponentCreator('/docs/css/typography', 'dd7'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/css/variables',
                component: ComponentCreator('/docs/css/variables', 'a11'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/devtools',
                component: ComponentCreator('/docs/devtools', '3eb'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/getting-started/installation',
                component: ComponentCreator('/docs/getting-started/installation', '775'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/getting-started/overview',
                component: ComponentCreator('/docs/getting-started/overview', '537'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/getting-started/quick-start',
                component: ComponentCreator('/docs/getting-started/quick-start', '6c2'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/html/dom-api',
                component: ComponentCreator('/docs/html/dom-api', '0f3'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/html/element-index',
                component: ComponentCreator('/docs/html/element-index', '854'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/html/elements',
                component: ComponentCreator('/docs/html/elements', 'ec9'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/html/overview',
                component: ComponentCreator('/docs/html/overview', '0fe'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/html/parsing',
                component: ComponentCreator('/docs/html/parsing', 'f6c'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/interactivity/events',
                component: ComponentCreator('/docs/interactivity/events', '6e6'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/interactivity/focus-keyboard',
                component: ComponentCreator('/docs/interactivity/focus-keyboard', '5ad'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/interactivity/forms',
                component: ComponentCreator('/docs/interactivity/forms', '925'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/interactivity/overview',
                component: ComponentCreator('/docs/interactivity/overview', '9be'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/interactivity/scrolling',
                component: ComponentCreator('/docs/interactivity/scrolling', '324'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/interactivity/text-selection',
                component: ComponentCreator('/docs/interactivity/text-selection', 'b9a'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/layout/block',
                component: ComponentCreator('/docs/layout/block', '72e'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/layout/flexbox',
                component: ComponentCreator('/docs/layout/flexbox', 'a15'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/layout/grid',
                component: ComponentCreator('/docs/layout/grid', '446'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/layout/inline',
                component: ComponentCreator('/docs/layout/inline', '51d'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/layout/overview',
                component: ComponentCreator('/docs/layout/overview', '43f'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/layout/positioned',
                component: ComponentCreator('/docs/layout/positioned', '1d3'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/rendering/overview',
                component: ComponentCreator('/docs/rendering/overview', 'e96'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/rendering/painting',
                component: ComponentCreator('/docs/rendering/painting', 'ab8'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/rendering/pipelines',
                component: ComponentCreator('/docs/rendering/pipelines', 'a2d'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/rust-integration/egui-backend',
                component: ComponentCreator('/docs/rust-integration/egui-backend', 'ebd'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/rust-integration/integrating',
                component: ComponentCreator('/docs/rust-integration/integrating', '7f6'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/rust-integration/overview',
                component: ComponentCreator('/docs/rust-integration/overview', '58d'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/rust-integration/profiling',
                component: ComponentCreator('/docs/rust-integration/profiling', '0c1'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/rust-integration/screenshots',
                component: ComponentCreator('/docs/rust-integration/screenshots', 'af6'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/rust-integration/winit-harness',
                component: ComponentCreator('/docs/rust-integration/winit-harness', 'ab0'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/status',
                component: ComponentCreator('/docs/status', '7c5'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/text/fonts',
                component: ComponentCreator('/docs/text/fonts', 'b35'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/text/overview',
                component: ComponentCreator('/docs/text/overview', '907'),
                exact: true,
                sidebar: "docs"
              },
              {
                path: '/docs/text/shaping',
                component: ComponentCreator('/docs/text/shaping', '8d9'),
                exact: true,
                sidebar: "docs"
              }
            ]
          }
        ]
      }
    ]
  },
  {
    path: '*',
    component: ComponentCreator('*'),
  },
];
