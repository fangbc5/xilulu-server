import { defineConfig, loadEnv } from 'vite'
import tailwindcss from '@tailwindcss/vite'
import vue from '@vitejs/plugin-vue'
import vueJsx from '@vitejs/plugin-vue-jsx'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { NaiveUiResolver } from 'unplugin-vue-components/resolvers'
import { browserslistToTargets } from 'lightningcss'
import browserslist from 'browserslist'
import { resolve } from 'path'

export default defineConfig(({ mode }) => {
  const root = process.cwd();

  function pathResolve(dir: string) {
    return resolve(root, '.', dir);
  }

  const env = loadEnv(mode, root)
  return {
    css: {
      transformer: 'lightningcss',
      lightningcss: {
        targets: browserslistToTargets(browserslist('>=0.25%')),
      },
    },
    plugins: [
      vue(),
      vueJsx(),
      tailwindcss(),
      AutoImport({
        imports: [
          'vue',
          'vue-router',
          'pinia',
          {
            'naive-ui': [
              'useDialog',
              'useMessage',
              'useNotification',
              'useLoadingBar'
            ]
          }
        ]
      }),
      Components({
        resolvers: [NaiveUiResolver()]
      }),
    ],
    resolve: {
      alias: [
        {
          find: /\/#\//,
          replacement: pathResolve('src/types') + '/',
        },
        {
          find: '@',
          replacement: pathResolve('src') + '/',
        },
      ]
    },
    base: env.VITE_BASE_URL || '/',
    build: {
      cssMinify: 'lightningcss',
      assetsDir: 'assets',
    },
    server: {
      host: '0.0.0.0',
      port: parseInt(env.VITE_PORT) || 5174,
      proxy: {
        '/api': {
          target: env.VITE_PROXY_URL || 'http://localhost:30002',
          changeOrigin: true,
          rewrite: (path) => path,
        }
      }
    }
  }
})

