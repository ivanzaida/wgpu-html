// @ts-check
import { themes as prismThemes } from 'prism-react-renderer';

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'wgpu-html',
  tagline: 'GPU-accelerated HTML/CSS renderer for Rust',
  favicon: 'img/favicon.ico',

  url: 'https://wgpu-html.dev',
  baseUrl: '/',

  organizationName: 'wgpu-html',
  projectName: 'wgpu-html',

  onBrokenLinks: 'warn',
  onBrokenMarkdownLinks: 'warn',

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          path: 'docs',
          routeBasePath: 'docs',
          sidebarPath: require.resolve('./sidebars.js'),
          editUrl: 'https://github.com/wgpu-html/wgpu-html/tree/master/www/',
          showLastUpdateTime: true,
        },
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      colorMode: {
        defaultMode: 'dark',
        disableSwitch: false,
        respectPrefersColorScheme: true,
      },
      navbar: {
        title: 'wgpu-html',
        logo: {
          alt: 'wgpu-html logo',
          src: 'img/logo.svg',
        },
        items: [
          {
            type: 'docSidebar',
            sidebarId: 'docs',
            position: 'left',
            label: 'Docs',
          },
          {
            href: 'https://github.com/wgpu-html/wgpu-html',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'Docs',
            items: [
              { label: 'Getting Started', to: '/docs/getting-started/overview' },
              { label: 'HTML Elements', to: '/docs/html/element-index' },
              { label: 'CSS Properties', to: '/docs/css/property-index' },
              { label: 'Rust Integration', to: '/docs/rust-integration/overview' },
            ],
          },
          {
            title: 'Community',
            items: [
              { label: 'GitHub', href: 'https://github.com/wgpu-html/wgpu-html' },
            ],
          },
          {
            title: 'More',
            items: [
              { label: 'Status', to: '/docs/status' },
              { label: 'vs RmlUI', to: '/docs/comparison-wgpu-html-vs-rmlui' },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} wgpu-html contributors. Built with Docusaurus.`,
      },
      prism: {
        theme: prismThemes.github,
        darkTheme: prismThemes.dracula,
        additionalLanguages: ['rust', 'toml', 'bash'],
      },
    }),
};

export default config;
