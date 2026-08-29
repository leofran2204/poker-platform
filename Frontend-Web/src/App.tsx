import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { Layout } from "@/components/Layout";
import { AdminAntifraudPage } from "@/pages/AdminAntifraudPage";
import { AdminAuditPage } from "@/pages/AdminAuditPage";
import { AdminClubsPage } from "@/pages/AdminClubsPage";
import { AdminDepositsPage } from "@/pages/AdminDepositsPage";
import { AdminLayout } from "@/pages/AdminLayout";
import { AdminOverviewPage } from "@/pages/AdminOverviewPage";
import { AdminPresencePage } from "@/pages/AdminPresencePage";
import { AdminTablesPage } from "@/pages/AdminTablesPage";
import { AdminTournamentsPage } from "@/pages/AdminTournamentsPage";
import { AdminUsersPage } from "@/pages/AdminUsersPage";
import { HomePage } from "@/pages/HomePage";
import { LobbyPage } from "@/pages/LobbyPage";
import { LoginPage } from "@/pages/LoginPage";
import { RegisterPage } from "@/pages/RegisterPage";
import { TablePage } from "@/pages/TablePage";
import { TournamentPage } from "@/pages/TournamentPage";
import { VerifyEmailPage } from "@/pages/VerifyEmailPage";
import { WalletPage } from "@/pages/WalletPage";

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<Layout />}>
          <Route index element={<HomePage />} />
          <Route path="login" element={<LoginPage />} />
          <Route path="register" element={<RegisterPage />} />
          <Route path="verify-email" element={<VerifyEmailPage />} />
          <Route path="lobby" element={<LobbyPage />} />
          <Route path="wallet" element={<WalletPage />} />
          <Route path="tournament/:id" element={<TournamentPage />} />
          <Route path="table/:id" element={<TablePage />} />
          <Route path="admin" element={<AdminLayout />}>
            <Route index element={<AdminOverviewPage />} />
            <Route path="users" element={<AdminUsersPage />} />
            <Route path="deposits" element={<AdminDepositsPage />} />
            <Route path="tables" element={<AdminTablesPage />} />
            <Route path="tournaments" element={<AdminTournamentsPage />} />
            <Route path="presence" element={<AdminPresencePage />} />
            <Route path="clubs" element={<AdminClubsPage />} />
            <Route path="antifraud" element={<AdminAntifraudPage />} />
            <Route path="audit" element={<AdminAuditPage />} />
          </Route>
          <Route path="*" element={<Navigate to="/" replace />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
