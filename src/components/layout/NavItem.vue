<script setup lang="ts">
import { computed, type Component } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Tooltip } from 'ant-design-vue'

const props = defineProps<{
  icon: Component
  label: string
  to: string
  collapsed: boolean
}>()

const route = useRoute()
const router = useRouter()

const isActive = computed(() => route.path === props.to)

function navigate() {
  router.push(props.to)
}
</script>

<template>
  <Tooltip :title="collapsed ? label : ''" placement="right" :mouse-enter-delay="0.3">
    <div
      class="nav-item"
      :class="{ 'nav-item--active': isActive }"
      @click="navigate"
    >
      <span class="nav-item-icon">
        <component :is="icon" />
      </span>
      <span class="nav-item-label">{{ label }}</span>
    </div>
  </Tooltip>
</template>
