<template>
  <div class="min-h-screen flex bg-slate-50">
    <!-- Sidebar -->
    <aside class="w-56 bg-slate-900 text-white flex flex-col">
      <div class="px-4 py-4 font-bold text-lg border-b border-slate-700">
        Sudut Buku
      </div>
      <nav class="flex-1 px-2 py-4 space-y-1 text-sm">
        <RouterLink
          to="/dashboard"
          class="block rounded-lg px-3 py-2 hover:bg-slate-800"
        >
          Dashboard
        </RouterLink>
        <RouterLink
          to="/books"
          class="block rounded-lg px-3 py-2 hover:bg-slate-800"
        >
          Books
        </RouterLink>
        <RouterLink
          to="/members"
          class="block rounded-lg px-3 py-2 hover:bg-slate-800"
        >
          Members
        </RouterLink>
        <RouterLink
          to="/loans"
          class="block rounded-lg px-3 py-2 hover:bg-slate-800"
        >
          Loans
        </RouterLink>
      </nav>
    </aside>

    <!-- Main content -->
    <div class="flex-1 flex flex-col">
      <!-- Topbar -->
      <header class="h-14 flex items-center justify-between px-6 border-b bg-white">
        <h1 class="font-semibold text-lg">
          {{ title }}
        </h1>
        <div class="text-sm text-slate-500">
          Admin
        </div>
      </header>

      <!-- Page body -->
      <main class="flex-1 p-6">
        <slot />
      </main>
    </div>
  </div>
</template>

<script setup>
const props = defineProps({
  title: {
    type: String,
    default: 'Dashboard',
  },
})
</script>