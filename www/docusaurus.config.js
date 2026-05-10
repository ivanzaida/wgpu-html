// @ts-check
import {themes as prismThemes} from 'prism-react-renderer';

/** @type {import('@docusaurus/types').Config} */
const config = {
    title: 'wgpu-html',
    tagline: 'GPU-accelerated HTML/CSS renderer for Rust',
    favicon: 'img/favicon.ico',

    url: 'https://ivanzaida.github.io',
    baseUrl: '/wgpu-html/',

    organizationName: 'ivanzaida',
    projectName: 'wgpu-html',

    onBrokenLinks: 'warn',

    markdown: {
        hooks: {
            onBrokenMarkdownLinks: 'warn',
        },
    },

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
                    editUrl: 'https://github.com/ivanzaida/wgpu-html/tree/master/www/',
                    showLastUpdateTime: true,
                },
                blog: false,
                theme: {
                    customCss: require.resolve('./src/css/custom.css'),
                },
            }),
        ],
    ],

    plugins: [
        [
            '@easyops-cn/docusaurus-search-local',
            {
                hashed: true,
                language: ['en'],
                indexDocs: true,
                indexPages: true,
                docsRouteBasePath: 'docs',
                highlightSearchTermsOnTargetPage: true,
            },
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
                            {label: 'Getting Started', to: '/docs/getting-started/overview'},
                            {label: 'Supported HTML', to: '/docs/features/supported-html'},
                            {label: 'Supported CSS', to: '/docs/features/supported-css'},
                            {label: 'Engine Architecture', to: '/docs/engine/architecture'},
                        ],
                    },
                    {
                        title: 'Community',
                        items: [
                            {label: 'GitHub', href: 'https://github.com/wgpu-html/wgpu-html'},
                        ],
                    },
                    {
                        title: 'More',
                        items: [
                            {label: 'Roadmap', to: '/docs/roadmap/'},
                            {label: 'Contributing', to: '/docs/development/contributing'},
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
