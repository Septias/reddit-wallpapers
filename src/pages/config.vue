<script lang="ts" setup async>
import { invoke } from '@tauri-apps/api/tauri'

interface Config {
  path: string
  username: string
  password: string
  client_id: string
  client_secret: string
}

const reference = reactive(await invoke('get_config') as Config)
const first_setup = ref(reference.username === '')
const config = reactive({ ...reference })
const err = ref('')
const router = useRouter()

async function save() {
  try {
    await invoke('set_config', { newConfig: config })
    router.push('/')
  }
  catch (e: any) {
    console.log(e)
    err.value = e
  }
  first_setup.value = false
  Object.assign(reference, config)
}

const is_equal = computed(() => JSON.stringify(reference) === JSON.stringify(config))
</script>

<template lang="pug">
.flex.justify-center.items-center.h-screen
  .border.rounded-xl.p-2.text-white.flex.flex-col.custom-width.w-max
    div.flex.justify-between.items-center
      h1.text-xl.font-bold Config
      button.bg-rose-500.px-2.py-1.self-end.rounded.leading-none(@click="save" v-if="!is_equal") Save
    p.text-red(v-if="err") {{ err }}
    label username
    input.input.mb-2(v-model="config.username")
    label password
    input.input.mb-2(v-model="config.password")
    label path
    input.input.mb-2(v-model="config.path" :contenteditable="first_setup")
    label client-id
    input.input.mb-2(v-model="config.client_id")
    label client-secret
    input.input.mb-2(v-model="config.client_secret")
</template>

<style lang="sass">
.custom-width
  width: 600px

.input
  @apply rounded bg-primaryl p-1 outline-none
</style>
