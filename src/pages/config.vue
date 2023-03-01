<script lang="ts" setup async>
import { invoke } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/api/dialog'

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
    console.log('login success')
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

async function handle_select_path() {
  config.path = await open({ directory: true })
}

</script>

<template lang="pug">
.flex.justify-center.items-center.h-screen
  router-link.absolute.top-0.left-0.bg-primaryl.p-1.rounded.m-1(to="/")
    div.text-white.i-carbon-home
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
    .flex.gap-2
      input.input.mb-2.flex-grow(v-model="config.path" :disabled="!first_setup")
      button.p-1.leading-none.bg-primaryl.i-carbon-select-02(@click="handle_select_path")
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
