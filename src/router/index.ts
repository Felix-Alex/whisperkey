import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/settings/general',
    },
    {
      path: '/settings',
      redirect: '/settings/general',
    },
    {
      path: '/settings/general',
      name: 'General',
      component: () => import('../views/Settings/General.vue'),
    },
    {
      path: '/settings/hotkey',
      name: 'Hotkey',
      component: () => import('../views/Settings/Hotkey.vue'),
    },
    {
      path: '/settings/providers',
      name: 'Providers',
      component: () => import('../views/Settings/Providers.vue'),
    },
    {
      path: '/settings/activation',
      name: 'Activation',
      component: () => import('../views/Settings/Activation.vue'),
    },
    {
      path: '/settings/history',
      name: 'History',
      component: () => import('../views/Settings/History.vue'),
    },
    {
      path: '/settings/about',
      name: 'About',
      component: () => import('../views/Settings/About.vue'),
    },
    {
      path: '/indicator',
      name: 'Indicator',
      component: () => import('../views/Indicator/RecordIndicator.vue'),
    },
    {
      path: '/activate',
      name: 'ActivateDialog',
      component: () => import('../views/Activate/ActivateDialog.vue'),
    },
  ],
})

export default router
