<script setup lang="ts">
defineProps<{
  title: string
}>()
</script>

<template>
  <div class="base-page">
    <header class="base-page-header">
      <h2 class="base-page-title">{{ title }}</h2>
      <div class="base-page-actions">
        <slot name="actions" />
      </div>
    </header>
    <div class="base-page-content">
      <slot />
    </div>
  </div>
</template>
