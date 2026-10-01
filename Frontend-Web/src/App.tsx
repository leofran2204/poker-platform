import { lazy, Suspense } from "react";
import { BrowserRouter, Navigate, Route, Routes } from "react-router";
import { Layout } from "@/components/Layout";
import { EstruturaPage } from "@/pages/EstruturaPage";
import { HomePage } from "@/pages/HomePage";
import { LobbyPage } from "@/pages/LobbyPage";
import { LoginPage } from "@/pages/LoginPage";
import { ForgotPasswordPage } from "@/pages/ForgotPasswordPage";
import { KycPage } from "@/pages/KycPage";
import { RegisterPage } from "@/pages/RegisterPage";
import { RedePage } from "@/pages/RedePage";
import { ResponsibleGamingPage } from "@/pages/ResponsibleGamingPage";
import { SupportPage } from "@/pages/SupportPage";
import { TermsPage } from "@/pages/TermsPage";
import { VerifyEmailPage } from "@/pages/VerifyEmailPage";
import { WalletPage } from "@/pages/WalletPage";

const CoursePage = lazy(() => import("@/pages/CoursePage").then(m => ({ default: m.CoursePage })));
const LessonPage = lazy(() => import("@/pages/LessonPage").then(m => ({ default: m.LessonPage })));
const TrainingPage = lazy(() => import("@/pages/TrainingPage").then(m => ({ default: m.TrainingPage })));
const TablePage = lazy(() => import("@/pages/TablePage").then(m => ({ default: m.TablePage })));
const TournamentPage = lazy(() => import("@/pages/TournamentPage").then(m => ({ default: m.TournamentPage })));

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
const AdminProtectionPage = lazy(() => import("@/pages/AdminProtectionPage"));
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
      Carregando…
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
          <Route path="recuperar-senha" element={<ForgotPasswordPage />} />
          <Route path="register" element={<RegisterPage />} />
          <Route path="rede" element={<RedePage />} />
          <Route path="jogo-responsavel" element={<ResponsibleGamingPage />} />
          <Route path="verificacao" element={<KycPage />} />
          <Route path="suporte" element={<SupportPage />} />
          <Route path="termos" element={<TermsPage />} />
          <Route path="verify-email" element={<VerifyEmailPage />} />
          <Route path="lobby" element={<LobbyPage />} />
          <Route path="curso" element={<Suspense fallback={<AdminFallback />}><CoursePage /></Suspense>} />
          <Route path="curso/:lessonId" element={<Suspense fallback={<AdminFallback />}><LessonPage /></Suspense>} />
          <Route path="curso/mesa/:moduleId" element={<Suspense fallback={<AdminFallback />}><TrainingPage /></Suspense>} />
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
          <Route path="tournament/:id" element={<Suspense fallback={<AdminFallback />}><TournamentPage /></Suspense>} />
          <Route path="table/:id" element={<Suspense fallback={<AdminFallback />}><TablePage /></Suspense>} />
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
            <Route path="protection" element={<AdminProtectionPage />} />
          </Route>
          <Route path="*" element={<Navigate to="/" replace />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
