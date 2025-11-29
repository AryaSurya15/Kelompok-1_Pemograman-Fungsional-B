<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import AdminLayout from '../components/AdminLayout.vue'

interface Book {
  id: number
  title: string
  author: string
  category: string
  year: number
  total_copies: number
  available_copies: number
}

interface Member {
  id: number
  name: string
  email: string
  joined_at: string
}

interface Loan {
  id: number
  book_id: number
  member_id: number
  borrowed_at: string
  due_at: string
  returned_at: string | null
}

const books = ref<Book[]>([])
const members = ref<Member[]>([])
const loans = ref<Loan[]>([])

const loading = ref(false)
const error = ref<string | null>(null)

// Form peminjaman baru
const showAddForm = ref(false)
const selectedMemberId = ref<number | null>(null)
const selectedBookId = ref<number | null>(null)
const dueDate = ref('') // "YYYY-MM-DD"
const saving = ref(false)

// tanggal minimal (hari ini) untuk input date
const today = computed(() => {
  const d = new Date()
  const yyyy = d.getFullYear()
  const mm = String(d.getMonth() + 1).padStart(2, '0')
  const dd = String(d.getDate()).padStart(2, '0')
  return `${yyyy}-${mm}-${dd}` // "YYYY-MM-DD"
})

// ---------------- Helper ----------------

const availableBooks = computed(() =>
  books.value.filter((b) => b.available_copies > 0),
)

const enrichedLoans = computed(() => {
  const now = new Date()

  return loans.value.map((loan) => {
    const book = books.value.find((b) => b.id === loan.book_id)
    const member = members.value.find((m) => m.id === loan.member_id)

    const due = new Date(loan.due_at)
    const returned = loan.returned_at ? new Date(loan.returned_at) : null

    let status = 'Dipinjam'
    let isLate = false

    if (returned) {
      status = 'Selesai'
    } else if (now > due) {
      status = 'Terlambat'
      isLate = true
    }

    return {
      ...loan,
      bookTitle: book ? book.title : `#${loan.book_id}`,
      memberName: member ? member.name : `#${loan.member_id}`,
      status,
      isLate,
    }
  })
})

function formatDate(dt: string): string {
  if (!dt) return '-'
  const d = new Date(dt)
  if (isNaN(d.getTime())) return dt
  return d.toLocaleString()
}

// ---------------- Load data ----------------

async function loadAll() {
  loading.value = true
  error.value = null
  try {
    const [booksRes, membersRes, loansRes] = await Promise.all([
      fetch('http://127.0.0.1:8000/books'),
      fetch('http://127.0.0.1:8000/members'),
      fetch('http://127.0.0.1:8000/loans'),
    ])

    if (!booksRes.ok) throw new Error('Gagal memuat books')
    if (!membersRes.ok) throw new Error('Gagal memuat members')
    if (!loansRes.ok) throw new Error('Gagal memuat loans')

    books.value = await booksRes.json()
    members.value = await membersRes.json()
    loans.value = await loansRes.json()
  } catch (e: any) {
    error.value = e.message ?? 'Gagal memuat data peminjaman'
  } finally {
    loading.value = false
  }
}

// ---------------- Actions ----------------

async function createLoan() {
  if (!selectedMemberId.value || !selectedBookId.value || !dueDate.value) {
    error.value = 'Pilih anggota, buku, dan tanggal jatuh tempo'
    return
  }

  saving.value = true
  error.value = null

  try {
    const res = await fetch('http://127.0.0.1:8000/loans', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        member_id: selectedMemberId.value,
        book_id: selectedBookId.value,
        due_date: dueDate.value, // format "YYYY-MM-DD"
      }),
    })
    if (!res.ok) throw new Error(`HTTP ${res.status}`)

    const created: Loan = await res.json()

    // backend pakai id = -1 untuk menandai gagal (stok habis / tanggal tidak valid)
    if (created.id < 0) {
      error.value =
        'Peminjaman gagal: stok buku habis atau tanggal jatuh tempo tidak valid'
      return
    }

    // Berhasil → reload semua data supaya stok & loans sinkron
    await loadAll()

    // reset form
    selectedMemberId.value = null
    selectedBookId.value = null
    dueDate.value = ''
    showAddForm.value = false
  } catch (e: any) {
    error.value = e.message ?? 'Gagal membuat peminjaman'
  } finally {
    saving.value = false
  }
}

async function returnLoan(id: number) {
  const ok = confirm(`Tandai peminjaman #${id} sebagai dikembalikan?`)
  if (!ok) return

  try {
    const res = await fetch(`http://127.0.0.1:8000/loans/${id}/return`, {
      method: 'POST',
    })
    if (!res.ok) throw new Error(`HTTP ${res.status}`)
    const success: boolean = await res.json()
    if (!success) {
      alert('Gagal mengembalikan buku')
      return
    }

    // reload semua data dari backend
    await loadAll()
  } catch (e: any) {
    error.value = e.message ?? 'Gagal mengembalikan buku'
  }
}

onMounted(() => {
  loadAll()
})
</script>

<template>
  <AdminLayout title="Loans">
    <div class="space-y-4">
      <!-- Header -->
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-lg font-semibold">Data Peminjaman</h2>
          <p class="text-xs text-gray-500">
            Catatan siapa meminjam buku apa, kapan, dan statusnya.
          </p>
        </div>
        <button
          class="px-3 py-2 text-sm rounded bg-indigo-600 text-white hover:bg-indigo-700"
          @click="showAddForm = true"
        >
          + Peminjaman Baru
        </button>
      </div>

      <p v-if="error" class="text-sm text-red-600">{{ error }}</p>

      <!-- Tabel loans -->
      <section class="bg-white rounded border border-gray-200 overflow-hidden">
        <div class="px-3 py-2 border-b border-gray-200 flex justify-between items-center">
          <span class="text-sm font-semibold">Daftar Peminjaman</span>
          <span class="text-xs text-gray-500">
            {{ enrichedLoans.length }} peminjaman
          </span>
        </div>

        <table class="w-full border-collapse text-xs">
          <thead class="bg-gray-100 text-left">
            <tr>
              <th class="border p-2">ID</th>
              <th class="border p-2">Anggota</th>
              <th class="border p-2">Buku</th>
              <th class="border p-2">Dipinjam</th>
              <th class="border p-2">Jatuh Tempo</th>
              <th class="border p-2">Dikembalikan</th>
              <th class="border p-2">Status</th>
              <th class="border p-2">Aksi</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="loan in enrichedLoans"
              :key="loan.id"
              class="hover:bg-gray-50"
            >
              <td class="border p-2">{{ loan.id }}</td>
              <td class="border p-2">{{ loan.memberName }}</td>
              <td class="border p-2">{{ loan.bookTitle }}</td>
              <td class="border p-2">{{ formatDate(loan.borrowed_at) }}</td>
              <td class="border p-2">{{ formatDate(loan.due_at) }}</td>
              <td class="border p-2">
                {{ loan.returned_at ? formatDate(loan.returned_at) : '-' }}
              </td>
              <td class="border p-2">
                <span
                  :class="[
                    'px-2 py-0.5 rounded-full text-[10px] font-medium',
                    loan.status === 'Selesai'
                      ? 'bg-emerald-100 text-emerald-700'
                      : loan.isLate
                      ? 'bg-red-100 text-red-700'
                      : 'bg-amber-100 text-amber-700',
                  ]"
                >
                  {{ loan.status }}
                </span>
              </td>
              <td class="border p-2">
                <button
                  v-if="!loan.returned_at"
                  class="px-2 py-1 text-[11px] rounded bg-emerald-600 text-white hover:bg-emerald-700"
                  @click="returnLoan(loan.id)"
                >
                  Kembalikan
                </button>
                <span
                  v-else
                  class="text-[11px] text-gray-400"
                >
                  Selesai
                </span>
              </td>
            </tr>

            <tr v-if="!loading && enrichedLoans.length === 0">
              <td colspan="8" class="border p-2 text-center text-gray-500">
                Belum ada peminjaman.
              </td>
            </tr>
          </tbody>
        </table>
      </section>

      <!-- Form peminjaman baru -->
      <div
        v-if="showAddForm"
        class="fixed inset-0 bg-black/40 flex items-center justify-center"
      >
        <div class="bg-white rounded shadow-lg w-full max-w-md p-4 space-y-3 text-xs">
          <h3 class="text-sm font-semibold mb-1">Peminjaman Baru</h3>

          <div class="space-y-2">
            <div>
              <label class="block mb-1">Anggota</label>
              <select
                v-model.number="selectedMemberId"
                class="w-full border border-gray-300 rounded px-2 py-1"
              >
                <option :value="null" disabled>Pilih anggota...</option>
                <option
                  v-for="m in members"
                  :key="m.id"
                  :value="m.id"
                >
                  {{ m.name }} ({{ m.email }})
                </option>
              </select>
            </div>

            <div>
              <label class="block mb-1">Buku</label>
              <select
                v-model.number="selectedBookId"
                class="w-full border border-gray-300 rounded px-2 py-1"
              >
                <option :value="null" disabled>Pilih buku (tersedia)...</option>
                <option
                  v-for="b in availableBooks"
                  :key="b.id"
                  :value="b.id"
                >
                  {{ b.title }} – {{ b.author }} (sisa {{ b.available_copies }})
                </option>
              </select>
              <p class="text-[10px] text-gray-500 mt-1">
                Hanya buku dengan <b>stok &gt; 0</b> (available_copies) yang bisa dipinjam.
              </p>
            </div>

            <div>
              <label class="block mb-1">Tanggal Jatuh Tempo</label>
              <input
                v-model="dueDate"
                type="date"
                :min="today"
                class="w-full border border-gray-300 rounded px-2 py-1"
              />
            </div>
          </div>

          <div class="flex justify-end gap-2 mt-3">
            <button
              class="px-3 py-1.5 rounded border border-gray-300"
              @click="showAddForm = false"
              :disabled="saving"
            >
              Batal
            </button>
            <button
              class="px-3 py-1.5 rounded bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50"
              @click="createLoan"
              :disabled="saving"
            >
              {{ saving ? 'Menyimpan...' : 'Simpan' }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </AdminLayout>
</template>