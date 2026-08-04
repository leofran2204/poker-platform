import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { Layout } from "@/components/Layout";
import { AdminClubsPage } from "@/pages/AdminClubsPage";
import { HomePage } from "@/pages/HomePage";
import { LobbyPage } from "@/pages/LobbyPage";
import { LoginPage } from "@/pages/LoginPage";
import { RegisterPage } from "@/pages/RegisterPage";
import { TablePage } from "@/pages/TablePage";

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<Layout />}>
          <Route index element={<HomePage />} />
          <Route path="login" element={<LoginPage />} />
          <Route path="register" element={<RegisterPage />} />
          <Route path="lobby" element={<LobbyPage />} />
          <Route path="table/:id" element={<TablePage />} />
          <Route path="admin/clubs" element={<AdminClubsPage />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
