import { svelte } from '@sveltejs/vite-plugin-svelte'
import { resolve } from 'path'
import { defineConfig } from 'vite'

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [svelte()],
    build: {
        rollupOptions: {
            input: {
                dash: resolve(__dirname, 'html/dash.html'),
                food: resolve(__dirname, 'html/food.html'),
                weight: resolve(__dirname, 'html/weight.html'),
                users: resolve(__dirname, 'html/users.html'),
                login: resolve(__dirname, 'html/login.html'),
            },
            // output: {
            //     manualChunks: (id: any) => {
            //         if (id.includes('node_modules')) {
            //             if (id.includes('@smui') || id.includes('material')) {
            //                 return 'vendor_mui'
            //             }
            //         }
            //     },
            // },
        },
    },
})
