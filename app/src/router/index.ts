import { createRouter, createWebHistory } from 'vue-router'
import DashboardView from '../views/DashboardView.vue'
import BooksView from '../views/BooksView.vue'
import MembersView from '../views/MembersView.vue'   // <-- ini penting
import LoansView from '../views/LoansView.vue'

const routes = [
  { path: '/', redirect: '/dashboard' },
  { path: '/dashboard', name: 'Dashboard', component: DashboardView },
  { path: '/books', name: 'Books', component: BooksView },
  { path: '/members', name: 'Members', component: MembersView },
  { path: '/loans', name: 'Loans', component: LoansView }, // <—
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
