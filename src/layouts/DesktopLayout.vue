<script setup lang="ts">
import { Layout, LayoutContent } from 'ant-design-vue'
import { useNavCollapse } from '../composables/useNavCollapse'
import TitleBar from '../components/layout/TitleBar.vue'
import NavSidebar from '../components/layout/NavSidebar.vue'
import PrintStatusOverlay from '../views/print/PrintStatusOverlay.vue'

const { collapsed, canToggle, toggle } = useNavCollapse()
</script>

<template>
  <Layout class="app-layout">
    <Layout class="layout-body" has-sider>
      <NavSidebar
        :collapsed="collapsed"
        :can-toggle="canToggle"
        @toggle="toggle"
      />
      <Layout class="layout-right">
        <TitleBar />
        <LayoutContent class="layout-content">
          <router-view />
        </LayoutContent>
      </Layout>
    </Layout>
    <PrintStatusOverlay />
  </Layout>
</template>
