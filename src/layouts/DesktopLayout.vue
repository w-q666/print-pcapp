<script setup lang="ts">
import { useNavCollapse } from '../composables/useNavCollapse'
import TitleBar from '../components/layout/TitleBar.vue'
import NavSidebar from '../components/layout/NavSidebar.vue'
import PrintStatusOverlay from '../views/print/PrintStatusOverlay.vue'

const { collapsed, canToggle, toggle } = useNavCollapse()
</script>

<template>
  <div class="app-layout">
    <div class="layout-body">
      <NavSidebar
        :collapsed="collapsed"
        :can-toggle="canToggle"
        @toggle="toggle"
      />
      <div class="layout-right">
        <TitleBar />
        <main class="layout-content">
          <router-view />
        </main>
      </div>
    </div>
    <PrintStatusOverlay />
  </div>
</template>
