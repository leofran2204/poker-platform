import { lazy, Suspense } from "react";
import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { Layout } from "@/components/Layout";
import { CoursePage } from "@/pages/CoursePage";
import { EstruturaPage } from "@/pages/EstruturaPage";
import { HomePage } from "@/pages/HomePage";
import { LessonPage } from "@/pages/LessonPage";
import { LobbyPage } from "@/pages/LobbyPage";
import { LoginPage } from "@/pages/LoginPage";
import { RegisterPage } from "@/pages/RegisterPage";
import { TablePage } from "@/pages/TablePage";
import { TournamentPage } from "@/pages/TournamentPage";
import { VerifyEmailPage } from "@/pages/VerifyEmailPage";
import { WalletPage } from "@/pages/WalletPage";

const AdminLayout = lazy(() =>
  import("@/pages/AdminLayout").then((m) => ({ default: m.AdminLayout })),
);
const AdminOverviewPage = lazy(() =>
  import("@/pages/AdminOverviewPage").then((m) => ({ default: m.AdminOverviewPage })),
);
const AdminUsersPage = lazy(() =>
  import("@/pages/AdminUsersPage").then((m) => ({ default: m.AdminUsersPage })),
);
const AdminDepositsPage = lazy(() =>
  import("@/pages/AdminDepositsPage").then((m) => ({ default: m.AdminDepositsPage })),
);
const AdminTablesPage = lazy(() =>
  import("@/pages/AdminTablesPage").then((m) => ({ default: m.AdminTablesPage })),
);
const AdminTournamentsPage = lazy(() =>
  import("@/pages/AdminTournamentsPage").then((m) => ({ default: m.AdminTournamentsPage })),
);
const AdminPresencePage = lazy(() =>
  import("@/pages/AdminPresencePage").then((m) => ({ default: m.AdminPresencePage })),
);
const AdminClubsPage = lazy(() =>
  import("@/pages/AdminClubsPage").then((m) => ({ default: m.AdminClubsPage })),
);
const AdminAntifraudPage = lazy(() =>
  import("@/pages/AdminAntifraudPage").then((m) => ({ default: m.AdminAntifraudPage })),
);
const AdminAuditPage = lazy(() =>
  import("@/pages/AdminAuditPage").then((m) => ({ default: m.AdminAuditPage })),
);
const AdminBotsPage = lazy(() =>
  import("@/pages/AdminBotsPage").then((m) => ({ default: m.AdminBotsPage })),
);
const NewsPage = lazy(() =>
  import("@/pages/NewsPage").then((m) => ({ default: m.NewsPage })),
);
const TipsPage = lazy(() =>
  import("@/pages/TipsPage").then((m) => ({ default: m.TipsPage })),
);

function AdminFallback() {
  return (
    <div className="flex items-center gap-3 p-8 text-sm text-felt-300">
      <span className="zt-spinner" aria-hidden />
      Carregando painel…
    </div>
  );
}

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
          <Route path="curso" element={<CoursePage />} />
          <Route path="curso/:lessonId" element={<LessonPage />} />
          <Route
            path="noticias"
            element={
              <Suspense fallback={<AdminFallback />}>
                <NewsPage />
              </Suspense>
            }
          />
          <Route
            path="dicas"
            element={
              <Suspense fallback={<AdminFallback />}>
                <TipsPage />
              </Suspense>
            }
          />
          <Route path="wallet" element={<WalletPage />} />
          <Route path="estrutura" element={<EstruturaPage />} />
          <Route path="tournament/:id" element={<TournamentPage />} />
          <Route path="table/:id" element={<TablePage />} />
          <Route
            path="admin"
            element={
              <Suspense fallback={<AdminFallback />}>
                <AdminLayout />
              </Suspense>
            }
          >
            <Route index element={<AdminOverviewPage />} />
            <Route path="users" element={<AdminUsersPage />} />
            <Route path="deposits" element={<AdminDepositsPage />} />
            <Route path="tables" element={<AdminTablesPage />} />
            <Route path="tournaments" element={<AdminTournamentsPage />} />
            <Route path="presence" element={<AdminPresencePage />} />
            <Route path="clubs" element={<AdminClubsPage />} />
            <Route path="antifraud" element={<AdminAntifraudPage />} />
            <Route path="audit" element={<AdminAuditPage />} />
            <Route path="bots" element={<AdminBotsPage />} />
          </Route>
          <Route path="*" element={<Navigate to="/" replace />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
