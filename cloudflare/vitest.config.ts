import { defineWorkersConfig } from '@cloudflare/vitest-pool-workers/config';

export default defineWorkersConfig({
  test: {
    include: ['test/**/*.test.ts'],
    poolOptions: {
      workers: {
        // Disable isolated storage for WebSocket hibernation compatibility
        // See: https://developers.cloudflare.com/workers/testing/vitest-integration/known-issues/#isolated-storage
        isolatedStorage: false,
        main: './src/test-entry.ts',
        miniflare: {
          compatibilityDate: '2024-01-01',
          durableObjects: {
            POPUP_SESSION: 'PopupSession',
          },
          bindings: {
            // Test environment variables
            POPUP_AUTH_SECRET: 'test-secret-token',
          },
        },
      },
    },
  },
});
