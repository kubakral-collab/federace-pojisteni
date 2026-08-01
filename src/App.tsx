import { FormEvent, useEffect, useRef, useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
import {
  ArrowLeft,
  Archive,
  ChevronLeft,
  ChevronRight,
  Check,
  CircleX,
  CreditCard,
  Database,
  Info,
  FileText,
  FolderArchive,
  HardDriveDownload,
  LayoutDashboard,
  LogIn,
  LogOut,
  Power,
  Pencil,
  Plus,
  Printer,
  Mail,
  Search,
  Save,
  Settings,
  ShieldCheck,
  TriangleAlert,
  UserPlus,
  Users,
  Upload,
  X,
} from "lucide-react";
import "./App.css";
import { SETTINGS_MODULES } from "./modules/settings";

const UPDATE_CHECK_ENABLED_KEY = "pojisteni.updateCheckEnabled";
const LAST_UPDATE_CHECK_KEY = "pojisteni.lastUpdateCheck";

type Screen = "Vstup" | "Přehled" | "Pojištěnci" | "Seznam" | "Přidat platbu" | "Doklady o zaplacení" | "Pojistné události" | "Přehled pro pojišťovnu" | "Nová pojistná událost" | "Příkaz k úhradě" | "Archiv" | "Správa záloh" | "Nastavení" | "O programu";

type Receipt = {
  id: number; memberRowId: number; memberIdentifier: string; paymentId: number;
  registrationNumber: string; memberName: string; insuranceYear: number;
  paidOn: string; issuedOn: string; amount: number; contractNumber: string;
  status: string; emailStatus: string; sentAt?: string; recipientEmail?: string; checksum: string;
};

type BackupInfo = {
  path: string;
  fileName: string;
  applicationVersion: string;
  schemaVersion: number;
  createdAt: string;
  memberCount: number;
  databaseSize: number;
  checksum: string;
  emergency: boolean;
  provider: string;
};

type FormOptions = {
  organizations: string[];
  lastRegistrationNumber: number;
  lastClient: string;
  annualAmounts: number[];
};

type TariffResult = {
  premium: number;
  months: number;
  insuredAmount: number;
};

type Member = {
  rowId: number;
  identifier?: string;
  code?: string;
  registrationNumber?: string;
  insured: string;
  personalId?: string;
  affiliation?: string;
  insuranceFrom?: string;
  insuranceTo?: string;
  actualTermination?: string;
  category?: string;
  loss?: string;
  annualPremium?: string;
  premium?: string;
  actualPayment?: string;
  note?: string;
  title?: string;
  lastName?: string;
  firstName?: string;
  city?: string;
  address?: string;
  postalCode?: string;
  country?: string;
  organization?: string;
  email?: string;
};

type MemberPage = {
  members: Member[];
  total: number;
  page: number;
  pageSize: number;
};

type DashboardInfo = {
  memberCount: number;
  lastRegistrationNumber: number;
  databaseDate: string;
  programVersion: string;
  activeInsuranceYear: number;
  commitSha: string;
  buildDate: string;
  gitTag: string;
  overdueCount: number;
  overdueAmount: number;
  oldestDueDate?: string;
};

type MemberFilters = {
  affiliation: string;
  category: string;
  loss: string;
  status: string;
  premium: string;
  payment: string;
  paymentStatus: string;
  overdue: string;
};

type ArchiveYear = {
  year: number;
  recordCount: number;
  uniqueMemberCount?: number;
};

type TariffRate = {
  id: number;
  insuredAmount: number;
  category: "A" | "B" | "C";
  lossInsurance: boolean;
  annualPremium: number;
  validFrom: string;
  validTo?: string;
  active: boolean;
  note?: string;
};

type TariffRateInput = Omit<TariffRate, "id"> & { id?: number };

type PaymentSettings = {
  recipientName: string;
  accountNumber: string;
  bankCode: string;
  iban: string;
  bic: string;
  constantSymbol: string;
  defaultDueDays: number;
  messageTemplate: string;
};

type EmailSettings = { server: string; port: number; username: string; senderEmail: string; encryption: string; credentialName: string; passwordConfigured: boolean; password?: string };
type ReceiptSettings = { automaticCreation: boolean; automaticSending: boolean; emailSubject: string; emailBody: string; policyholder: string; contractNumber: string };
type PaymentDocumentBasis = { memberRowId: number; memberName: string; registrationNumber: string; organizationCode: string; insuranceYear: number; prescribedPremium: number; paidAmount: number; paymentDates: string[]; contractNumber: string; insuranceStatus: string; lossInsurance: boolean; certificateReady: boolean };

type PaymentOrderDraft = {
  rowId: number;
  payerName: string;
  address: string;
  city: string;
  postalCode: string;
  registrationNumber: string;
  insuranceYear: number;
  insuredAmount: number;
  annualPremium: number;
  actualPayment: number;
  amountDue: number;
  organization: string;
  variableSymbol: string;
  recipientName: string;
  account: string;
  iban: string;
  bic: string;
  constantSymbol: string;
  issueDate: string;
  dueDate: string;
  message: string;
  settingsComplete: boolean;
  validationErrors: string[];
};

type Claim = {
  id: number;
  memberIdentifier: number;
  insuranceRowId: number;
  insuranceYear: number;
  occurredOn?: string;
  reportedOn?: string;
  assessedDamage?: number;
  insuranceBenefit?: number;
  description?: string;
  phone?: string; employer?: string; occupation?: string; note?: string;
  additionalInformation?: string; handledBy?: string; reportPosition?: string;
  closedOn?: string;
  status: "Otevřená" | "Uzavřená";
};

type ClaimOverview = {
  id: number; memberRowId: number; memberName: string; registrationNumber: string;
  organizationCode: string; insuranceYear: number; occurredOn?: string; reportedOn?: string;
  description?: string; assessedDamage?: number; insuranceBenefit?: number;
  status: "Otevřená" | "Uzavřená"; lastChanged: string;
};

type MemberPayment = {
  id: number;
  receivedOn: string;
  amount: number;
  insuranceYear: number;
  method: "Bankovní převod" | "Hotově" | "Jiné";
  variableSymbol: string;
  note?: string;
  status: string;
  importedFromBank: boolean;
  bankTransactionId?: string;
};

type MemberPaymentForm = {
  id?: number;
  insuranceRowId: number;
  receivedOn: string;
  amount: string;
  method: "Bankovní převod" | "Hotově" | "Jiné";
  note: string;
};

type ClaimForm = {
  insuranceRowId: number;
  phone: string;
  employer: string;
  occupation: string;
  occurredOn: string;
  reportedOn: string;
  assessedDamage: string;
  insuranceBenefit: string;
  description: string;
  note: string;
  additionalInformation: string;
  closedOn: string;
  handledBy: string;
  reportPosition: string;
};

type AuditEntry = {
  occurredAt: string;
  user: string;
  operation: string;
  result: "OK" | "ERROR" | string;
};

function emptyClaimForm(rowId: number): ClaimForm {
  return {
    insuranceRowId: rowId,
    phone: "",
    employer: "",
    occupation: "",
    occurredOn: "",
    reportedOn: "",
    assessedDamage: "",
    insuranceBenefit: "",
    description: "",
    note: "",
    additionalInformation: "",
    closedOn: "",
    handledBy: "MAXIMA pojišťovna, a.s.",
    reportPosition: "",
  };
}

type MemberUpdate = {
  rowId: number;
  title: string;
  lastName: string;
  firstName: string;
  personalId: string;
  registrationNumber: number | null;
  city: string;
  address: string;
  postalCode: string;
  country: string;
  organization: string;
  affiliation: string;
  code: string;
  email: string;
  note: string;
  actualPayment: number | null;
  actualTermination: string;
};

type InsuredForm = {
  title: string;
  lastName: string;
  firstName: string;
  personalId: string;
  organization: string;
  affiliation: "FVČ" | "FV";
  city: string;
  address: string;
  postalCode: string;
  country: string;
  note: string;
  insuranceFrom: string;
  insuranceTo: string;
  annualAmount: number;
  category: "A" | "B" | "C";
  loss: boolean;
  actualPayment: string;
  code: number;
  registrationYear: number;
  email: string;
};

const emptyFilters: MemberFilters = {
  affiliation: "",
  category: "",
  loss: "",
  status: "",
  premium: "",
  payment: "",
  paymentStatus: "",
  overdue: "",
};
const preview = import.meta.env.DEV
  ? new URLSearchParams(window.location.search).get("preview")
  : null;

function initialScreen(): Screen {
  if (preview === "prehled") return "Přehled";
  if (preview === "seznam") return "Seznam";
  if (preview === "archiv") return "Archiv";
  if (preview === "nastaveni") return "Nastavení";
  return "Vstup";
}

const previewMembers: MemberPage = {
  members: [
    {
      rowId: 1,
      code: "1",
      registrationNumber: "150",
      insured: "Testovací člen A",
      personalId: "000000/0000",
      affiliation: "FVČ",
      insuranceFrom: "2026-01-01 00:00:00",
      category: "B",
      loss: "0",
      annualPremium: "200000",
      premium: "495",
      actualPayment: "495",
      note: "",
    },
    {
      rowId: 2,
      code: "2",
      registrationNumber: "151",
      insured: "Testovací člen B",
      personalId: "000000/0001",
      affiliation: "FV",
      insuranceFrom: "2026-01-01 00:00:00",
      category: "A",
      loss: "-1",
      annualPremium: "280000",
      premium: "1427",
      actualPayment: "1427",
      note: "Kontrolní náhled rozhraní",
    },
  ],
  total: 14_416,
  page: 1,
  pageSize: 50,
};

function emptyForm(year: number): InsuredForm {
  return {
    title: "",
    lastName: "",
    firstName: "",
    personalId: "",
    organization: "",
    affiliation: "FVČ",
    city: "",
    address: "",
    postalCode: "",
    country: "",
    note: "",
    insuranceFrom: "",
    insuranceTo: year ? `${year}-12-31` : "",
    annualAmount: 200_000,
    category: "B",
    loss: false,
    actualPayment: "0",
    code: 1,
    registrationYear: year,
    email: "",
  };
}

function emptyTariffRate(): TariffRateInput {
  return {
    insuredAmount: 200_000,
    category: "B",
    lossInsurance: false,
    annualPremium: 0,
    validFrom: "",
    validTo: undefined,
    active: true,
    note: "",
  };
}

function optional(value: string): string | null {
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
}

function formatPersonalId(value: string): string {
  const digits = value.replace(/\D/g, "").slice(0, 10);
  return digits.length > 6 ? `${digits.slice(0, 6)}/${digits.slice(6)}` : digits;
}

function formatPostalCode(value: string): string {
  const digits = value.replace(/\D/g, "").slice(0, 5);
  return digits.length > 3 ? `${digits.slice(0, 3)} ${digits.slice(3)}` : digits;
}

function display(value?: string): string {
  return value === undefined || value === null || value === "" ? "—" : value;
}

function displayDate(value?: string): string {
  if (!value) return "—";
  const iso = value.match(/^(\d{4})-(\d{2})-(\d{2})/);
  if (iso) return `${Number(iso[3])}. ${Number(iso[2])}. ${iso[1]}`;
  const czech = value.match(/^(\d{1,2})\.(\d{1,2})\.(\d{4})$/);
  return czech ? `${Number(czech[1])}. ${Number(czech[2])}. ${czech[3]}` : value;
}

function displayDateTime(value?: string): string {
  if (!value) return "—";
  const match = value.match(/^(\d{4})-(\d{2})-(\d{2})[ T](\d{2}):(\d{2})/);
  return match ? `${Number(match[3])}. ${Number(match[2])}. ${match[1]} ${match[4]}:${match[5]}` : displayDate(value);
}

function displayCurrency(value?: string | number): string {
  if (value === undefined || value === null || value === "") return "—";
  const number = Number(value);
  return Number.isFinite(number)
    ? new Intl.NumberFormat("cs-CZ", {
        style: "currency",
        currency: "CZK",
        minimumFractionDigits: Number.isInteger(number) ? 0 : 2,
        maximumFractionDigits: 2,
      }).format(number)
    : String(value);
}

function displayBackupDate(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? value
    : new Intl.DateTimeFormat("cs-CZ", { dateStyle: "medium", timeStyle: "short" }).format(date);
}

function displayFileSize(value: number): string {
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toLocaleString("cs-CZ", { maximumFractionDigits: 1 })} kB`;
  return `${(value / 1024 / 1024).toLocaleString("cs-CZ", { maximumFractionDigits: 1 })} MB`;
}

function LossStatus({ value }: { value?: string }) {
  const active = value !== undefined && value !== null && Number(value) !== 0;
  return active
    ? <span className="loss-status active" aria-label="Ano"><Check /></span>
    : <span className="loss-status inactive" aria-label="Ne">—</span>;
}

function detailValue(member: Member, key: keyof Member, date?: boolean) {
  const value = member[key] as string | undefined;
  if (key === "loss") return <LossStatus value={value} />;
  if (key === "annualPremium" || key === "premium" || key === "actualPayment") {
    return displayCurrency(value);
  }
  return date ? displayDate(value) : display(value);
}

function insuranceYear(member: Member): string {
  return member.insuranceFrom?.match(/^(\d{4})/)?.[1] ?? "—";
}

function paymentSummary(member: Member) {
  const due = Number(member.premium ?? 0);
  const paid = Number(member.actualPayment ?? 0);
  const difference = paid - due;
  if (paid > due) {
    return { label: "Přeplatek", balanceLabel: "Přeplatek", balance: displayCurrency(difference), tone: "overpaid" };
  }
  if (paid >= due && due > 0) {
    return { label: "Uhrazeno", balanceLabel: "Zbývá uhradit", balance: "0 Kč", tone: "paid" };
  }
  if (paid > 0) {
    return { label: "Částečně uhrazeno", balanceLabel: "Zbývá uhradit", balance: displayCurrency(due - paid), tone: "partial" };
  }
  return { label: "Neuhrazeno", balanceLabel: "Zbývá uhradit", balance: displayCurrency(Math.max(due, 0)), tone: "unpaid" };
}

function hasPaymentDifference(member: Member): boolean {
  return Number(member.actualPayment ?? 0) !== Number(member.premium ?? 0);
}

function PaymentWarning({ member }: { member: Member }) {
  if (!hasPaymentDifference(member)) return null;
  return (
    <span className="payment-warning" title="Skutečně uhrazená částka neodpovídá předepsanému pojistnému." aria-label="Neuhrazeno">
      <TriangleAlert />
    </span>
  );
}

function DetailSection({
  title,
  rows,
}: {
  title: string;
  rows: Array<[string, React.ReactNode]>;
}) {
  return (
    <section className="detail-section">
      <h2>{title}</h2>
      <dl>
        {rows.map(([label, value]) => (
          <div key={label}><dt>{label}</dt><dd>{value}</dd></div>
        ))}
      </dl>
    </section>
  );
}

function MemberHeading({ member }: { member: Member }) {
  const payment = paymentSummary(member);
  const active = !member.actualTermination;
  return (
    <section className="detail-heading">
      <div>
        <h1>{display(member.insured)}</h1>
        <p>Evidenční číslo {display(member.registrationNumber)} · {display(member.affiliation)}</p>
      </div>
      <div className="detail-badges">
        <span className={active ? "insurance-active" : "insurance-ended"}>
          {active ? "Aktivní pojištění" : "Ukončené pojištění"}
        </span>
        <span className={`payment-${payment.tone}`}>
          {payment.tone === "paid" && <Check />} {payment.label}
        </span>
        {Number(member.loss ?? 0) !== 0 && <span className="insurance-active"><Check /> Pojištění ztráty</span>}
      </div>
    </section>
  );
}

function InsuranceSection({ member }: { member: Member }) {
  const payment = paymentSummary(member);
  return (
    <DetailSection title="Pojištění" rows={[
      ["Pojistný rok", insuranceYear(member)],
      ["Pojištění od", displayDate(member.insuranceFrom)],
      ["Pojištění do", displayDate(member.insuranceTo)],
      ["Skutečné ukončení", displayDate(member.actualTermination)],
      ["Pojistná částka", displayCurrency(member.annualPremium)],
      ["Roční pojistné", displayCurrency(member.premium)],
      ["Skutečně uhrazeno", displayCurrency(member.actualPayment)],
      [payment.balanceLabel, payment.balance],
      ["Stav úhrady", payment.label],
      ["Kategorie", display(member.category)],
      ["Pojištění ztráty", <LossStatus value={member.loss} />],
      ["Sazba použitá při výpočtu", displayCurrency(member.premium)],
      ["Základní organizace", display(member.organization)],
    ]} />
  );
}

function OverviewSection({ member }: { member: Member }) {
  const payment = paymentSummary(member);
  const overpayment = Math.max(Number(member.actualPayment ?? 0) - Number(member.premium ?? 0), 0);
  return <DetailSection title="Přehled člena" rows={[
    ["Evidenční číslo", display(member.registrationNumber)],
    ["Celé jméno", display(member.insured)],
    ["Rodné číslo", display(member.personalId)],
    ["Odborová příslušnost", display(member.affiliation)],
    ["OC", display(member.affiliation)],
    ["Kód OC", display(member.code)],
    ["Základní organizace", display(member.organization)],
    ["Kategorie", display(member.category)],
    ["Pojištění od", displayDate(member.insuranceFrom)],
    ["Ukončení pojištění", displayDate(member.actualTermination)],
    ["Roční pojistné", displayCurrency(member.premium)],
    ["Skutečně uhrazeno", displayCurrency(member.actualPayment)],
    ["Zbývá uhradit", displayCurrency(Math.max(Number(member.premium ?? 0) - Number(member.actualPayment ?? 0), 0))],
    ...(overpayment > 0 ? [["Přeplatek", displayCurrency(overpayment)] as [string, React.ReactNode]] : []),
    ["Pojistná částka", displayCurrency(member.annualPremium)],
    ["Pojištění ztráty", <LossStatus value={member.loss} />],
    ["Stav úhrady", payment.label],
  ]} />;
}

function PersonalSection({ member }: { member: Member }) {
  return <DetailSection title="Osobní údaje" rows={[
    ["Titul před jménem", display(member.title)],
    ["Jméno", display(member.firstName)],
    ["Příjmení", display(member.lastName)],
    ["Evidenční číslo", display(member.registrationNumber)],
    ["Rodné číslo", display(member.personalId)],
    ["Adresa", display(member.address)],
    ["Obec", display(member.city)],
    ["PSČ", display(member.postalCode)],
    ["Stát", display(member.country)],
  ]} />;
}

function OrganizationSection({ member }: { member: Member }) {
  return <DetailSection title="Organizace" rows={[
    ["Základní organizace", display(member.organization)],
    ["Odborová příslušnost", display(member.affiliation)],
    ["Kód OC", display(member.code)],
    ["Kategorie", display(member.category)],
    ["Pojištění od", displayDate(member.insuranceFrom)],
    ["Ukončení", displayDate(member.actualTermination)],
  ]} />;
}

function ContactSection({ member }: { member: Member }) {
  const fullAddress = [member.address, member.city, member.postalCode, member.country]
    .filter(Boolean)
    .join(", ");
  return <DetailSection title="Kontakt" rows={[
    ["Telefon", "—"],
    ["E-mail", member.email ? <span className="copyable-value">{member.email}</span> : "—"],
    ["Adresa", display(member.address)],
    ["Obec", display(member.city)],
    ["PSČ", display(member.postalCode)],
    ["Stát", display(member.country)],
    ["Celá adresa", fullAddress || "—"],
  ]} />;
}

function NotesSection({ member }: { member: Member }) {
  return <DetailSection title="Poznámky" rows={[
    ["Poznámka k pojištění", display(member.note)],
  ]} />;
}

function FilterBar({
  filters,
  onChange,
  onApply,
}: {
  filters: MemberFilters;
  onChange: (filters: MemberFilters) => void;
  onApply: () => void;
}) {
  const updateFilter = (key: keyof MemberFilters, value: string) =>
    onChange({ ...filters, [key]: value });
  return (
    <div className="member-filters">
      <select value={filters.affiliation} onChange={(event) => updateFilter("affiliation", event.target.value)} aria-label="Filtr odborové příslušnosti">
        <option value="">Odborová příslušnost: vše</option><option value="1">FVČ</option><option value="2">FV</option>
      </select>
      <select value={filters.category} onChange={(event) => updateFilter("category", event.target.value)} aria-label="Filtr Kategorie">
        <option value="">Kategorie: vše</option><option value="A">A</option><option value="B">B</option><option value="C">C</option>
      </select>
      <select value={filters.loss} onChange={(event) => updateFilter("loss", event.target.value)} aria-label="Filtr pojištění ztráty">
        <option value="">Pojištění ztráty: vše</option><option value="-1">Ano</option><option value="0">Ne</option>
      </select>
      <select value={filters.status} onChange={(event) => updateFilter("status", event.target.value)} aria-label="Filtr Aktivní nebo ukončený">
        <option value="">Stav: vše</option><option value="aktivni">Aktivní</option><option value="ukonceny">Ukončený</option>
      </select>
      <input value={filters.premium} onChange={(event) => updateFilter("premium", event.target.value.replace(/\D/g, ""))} placeholder="Pojistné (Kč)" aria-label="Filtr pojistného" />
      <input value={filters.payment} onChange={(event) => updateFilter("payment", event.target.value.replace(/\D/g, ""))} placeholder="Skutečně uhrazeno (Kč)" aria-label="Filtr skutečně uhrazené částky" />
      <select value={filters.paymentStatus} onChange={(event) => updateFilter("paymentStatus", event.target.value)} aria-label="Filtr stavu úhrady">
        <option value="">Úhrada: vše</option>
        <option value="uhrazeno">Uhrazeno</option>
        <option value="neuhrazeno">Neuhrazeno</option>
      </select>
      <select value={filters.overdue} onChange={(event) => updateFilter("overdue", event.target.value)} aria-label="Filtr splatnosti">
        <option value="">Splatnost: vše</option>
        <option value="po_splatnosti">Po splatnosti</option>
      </select>
      <button type="button" onClick={onApply}>Použít filtry</button>
      <button type="button" onClick={() => { onChange(emptyFilters); }}>Vymazat</button>
    </div>
  );
}

type ShellProps = {
  active: Exclude<Screen, "Vstup">;
  user: string;
  onNavigate: (screen: Exclude<Screen, "Vstup">) => void;
  onLogout: () => void;
  updater: {
    available: Update | null;
    installing: boolean;
    progress: number | null;
    message: string;
    onInstall: () => void;
    onLater: () => void;
  };
  backupBusy: boolean;
  onCreateBackup: () => void;
  onRestoreBackup: () => void;
  children: React.ReactNode;
};

function Shell({ active, user, onNavigate, onLogout, updater, backupBusy, onCreateBackup, onRestoreBackup, children }: ShellProps) {
  const [fileMenuOpen, setFileMenuOpen] = useState(false);
  const navigation: Array<{
    screen: Exclude<Screen, "Vstup">;
    label: string;
    icon: React.ReactNode;
  }> = [
    { screen: "Přehled", label: "Hlavní panel", icon: <LayoutDashboard /> },
    { screen: "Seznam", label: "Seznam pojištěnců", icon: <Users /> },
    { screen: "Přidat platbu", label: "Přidat platbu", icon: <CreditCard /> },
    { screen: "Doklady o zaplacení", label: "Doklady o zaplacení", icon: <FileText /> },
    { screen: "Příkaz k úhradě", label: "Příkazy k úhradě", icon: <FileText /> },
    { screen: "Pojistné události", label: "Pojistné události", icon: <TriangleAlert /> },
    { screen: "Přehled pro pojišťovnu", label: "Přehled pro pojišťovnu", icon: <LayoutDashboard /> },
    { screen: "Archiv", label: "Archiv", icon: <Archive /> },
    { screen: "Nastavení", label: "Nastavení", icon: <Settings /> },
    { screen: "Pojištěnci", label: "Nový pojištěnec", icon: <UserPlus /> },
    { screen: "Správa záloh", label: "Správa záloh", icon: <FolderArchive /> },
    { screen: "O programu", label: "O programu", icon: <Info /> },
  ];
  return (
    <div className="application-shell">
      <aside className="sidebar">
        <div className="sidebar-brand">
          <img src="/logo.png" alt="Federace vlakových čet" />
          <div>
            <strong>Pojištění</strong>
            <small>Federace vlakových čet</small>
          </div>
        </div>
        <div className="file-menu">
          <button className="file-menu-trigger" onClick={() => setFileMenuOpen((open) => !open)} disabled={backupBusy}>
            <FileText /> Soubor
          </button>
          {fileMenuOpen && (
            <div className="file-menu-popover">
              <button onClick={() => { setFileMenuOpen(false); onCreateBackup(); }}><HardDriveDownload /> Vytvořit zálohu…</button>
              <button onClick={() => { setFileMenuOpen(false); onRestoreBackup(); }}><Upload /> Obnovit ze zálohy…</button>
              <button onClick={() => { setFileMenuOpen(false); onNavigate("Správa záloh"); }}><FolderArchive /> Správa záloh</button>
            </div>
          )}
        </div>
        <nav>
          {navigation.map((item) => (
            <button
              key={item.screen}
              className={active === item.screen ? "active" : ""}
              onClick={() => onNavigate(item.screen)}
            >
              {item.icon}
              {item.label}
            </button>
          ))}
        </nav>
        <div className="sidebar-footer">
          <div className="signed-user">
            <small>Přihlášený uživatel</small>
            <strong>{user}</strong>
          </div>
          <button onClick={onLogout}><LogOut /> Odhlásit</button>
          <button onClick={() => invoke("quit_application")}><Power /> Konec</button>
        </div>
      </aside>
      <section className="content-area">{children}</section>
      {updater.available && (
        <div className="update-dialog-backdrop" role="presentation">
          <section className="update-dialog" role="dialog" aria-modal="true" aria-labelledby="update-dialog-title">
            <small>Aktualizace aplikace</small>
            <h2 id="update-dialog-title">Je dostupná nová verze</h2>
            <p><strong>Verze {updater.available.version}</strong></p>
            <div className="update-release-notes">
              <strong>Novinky</strong>
              <p>{updater.available.body?.trim() || "Podrobnosti k vydání nejsou uvedeny."}</p>
            </div>
            {updater.installing && (
              <div className="update-progress" aria-live="polite">
                <progress max="100" value={updater.progress ?? undefined} />
                <span>{updater.progress === null ? "Připravuji stažení…" : `Staženo ${updater.progress} %`}</span>
              </div>
            )}
            {updater.message && <div className="message error">{updater.message}</div>}
            <footer>
              <button className="primary" disabled={updater.installing} onClick={updater.onInstall}>Aktualizovat</button>
              <button disabled={updater.installing} onClick={updater.onLater}>Připomenout později</button>
            </footer>
          </section>
        </div>
      )}
    </div>
  );
}

export default function App() {
  const [screen, setScreen] = useState<Screen>(initialScreen);
  const [password, setPassword] = useState("");
  const [passwordConfirmation, setPasswordConfirmation] = useState("");
  const [authInitialized, setAuthInitialized] = useState<boolean | null>(preview ? true : null);
  const [user, setUser] = useState(preview ? "náhled" : "");
  const [role, setRole] = useState(preview ? "Správce" : "");
  const [form, setForm] = useState<InsuredForm>(() => emptyForm(0));
  const [options, setOptions] = useState<FormOptions>({
    organizations: [],
    lastRegistrationNumber: 0,
    lastClient: "",
    annualAmounts: [],
  });
  const [tariff, setTariff] = useState<TariffResult>({
    premium: 495,
    months: 0,
    insuredAmount: 0,
  });
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [saving, setSaving] = useState(false);
  const [memberPage, setMemberPage] = useState<MemberPage>(
    preview === "seznam"
      ? previewMembers
      : { members: [], total: 0, page: 1, pageSize: 50 },
  );
  const [memberSearch, setMemberSearch] = useState("");
  const [activeSearch, setActiveSearch] = useState("");
  const [memberFilters, setMemberFilters] = useState<MemberFilters>(emptyFilters);
  const [membersLoading, setMembersLoading] = useState(false);
  const [selectedMember, setSelectedMember] = useState<Member | null>(null);
  const [memberHistory, setMemberHistory] = useState<Member[]>([]);
  const [memberAuditHistory, setMemberAuditHistory] = useState<AuditEntry[]>([]);
  const [historyMember, setHistoryMember] = useState<Member | null>(null);
  const [detailTab, setDetailTab] = useState<
    "overview" | "personal" | "organization" | "contact" | "insurance" | "payments" | "receipts" | "claims" | "history" | "notes"
  >("overview");
  const [editingMember, setEditingMember] = useState(false);
  const [memberEdit, setMemberEdit] = useState<MemberUpdate | null>(null);
  const [archiveYears, setArchiveYears] = useState<ArchiveYear[]>([]);
  const [archiveYear, setArchiveYear] = useState<number | null>(null);
  const [archivePage, setArchivePage] = useState<MemberPage>({
    members: [],
    total: 0,
    page: 1,
    pageSize: 50,
  });
  const [archiveSearch, setArchiveSearch] = useState("");
  const [activeArchiveSearch, setActiveArchiveSearch] = useState("");
  const [archiveLoading, setArchiveLoading] = useState(false);
  const [selectedArchiveMember, setSelectedArchiveMember] = useState<Member | null>(null);
  const [archiveFilters, setArchiveFilters] = useState<MemberFilters>(emptyFilters);
  const [settingsSection, setSettingsSection] = useState<string | null>(null);
  const [updateCheckEnabled, setUpdateCheckEnabled] = useState(
    () => localStorage.getItem(UPDATE_CHECK_ENABLED_KEY) !== "false",
  );
  const [lastUpdateCheck, setLastUpdateCheck] = useState(
    () => localStorage.getItem(LAST_UPDATE_CHECK_KEY) ?? "Dosud neprovedena",
  );
  const [availableUpdate, setAvailableUpdate] = useState<Update | null>(null);
  const [updateChecking, setUpdateChecking] = useState(false);
  const [updateInstalling, setUpdateInstalling] = useState(false);
  const [updateProgress, setUpdateProgress] = useState<number | null>(null);
  const [updateMessage, setUpdateMessage] = useState("");
  const [updaterStatus, setUpdaterStatus] = useState("Dosud nezkontrolováno");
  const [backupBusy, setBackupBusy] = useState(false);
  const [backups, setBackups] = useState<BackupInfo[]>([]);
  const [backupsLoading, setBackupsLoading] = useState(false);
  const [tariffRates, setTariffRates] = useState<TariffRate[]>([]);
  const [tariffForm, setTariffForm] = useState<TariffRateInput | null>(null);
  const [tariffsLoading, setTariffsLoading] = useState(false);
  const [paymentSettings, setPaymentSettings] = useState<PaymentSettings | null>(null);
  const [emailSettings, setEmailSettings] = useState<EmailSettings | null>(null);
  const [receiptSettings, setReceiptSettings] = useState<ReceiptSettings | null>(null);
  const [paymentLoading, setPaymentLoading] = useState(false);
  const [paymentDraft, setPaymentDraft] = useState<PaymentOrderDraft | null>(null);
  const [lastPaymentPdf, setLastPaymentPdf] = useState("");
  const [returnToMember, setReturnToMember] = useState(false);
  const [memberClaims, setMemberClaims] = useState<Claim[]>([]);
  const [memberPayments, setMemberPayments] = useState<MemberPayment[]>([]);
  const [receipts, setReceipts] = useState<Receipt[]>([]);
  const [receiptSearch, setReceiptSearch] = useState("");
  const [memberReceipts, setMemberReceipts] = useState<Receipt[]>([]);
  const [paymentDocumentBasis, setPaymentDocumentBasis] = useState<PaymentDocumentBasis | null>(null);
  const [memberPaymentForm, setMemberPaymentForm] = useState<MemberPaymentForm | null>(null);
  const [claimsLoading, setClaimsLoading] = useState(false);
  const [claimForm, setClaimForm] = useState<ClaimForm | null>(null);
  const [createdClaimId, setCreatedClaimId] = useState<number | null>(null);
  const [editingClaimId, setEditingClaimId] = useState<number | null>(null);
  const [agendaSearch, setAgendaSearch] = useState("");
  const [agendaMembers, setAgendaMembers] = useState<Member[]>([]);
  const [agendaClaims, setAgendaClaims] = useState<ClaimOverview[]>([]);
  const [claimYearFilter, setClaimYearFilter] = useState("");
  const [claimStatusFilter, setClaimStatusFilter] = useState("");
  const [claimOcFilter, setClaimOcFilter] = useState("");
  const [dashboard, setDashboard] = useState<DashboardInfo | null>(
    preview
      ? {
          memberCount: 14_416,
          lastRegistrationNumber: 151,
          databaseDate: "26.07.2026",
          programVersion: "0.11.0",
          activeInsuranceYear: 2026,
          commitSha: "preview",
          buildDate: "2026-07-28T00:00:00Z",
          gitTag: "v0.17.0",
          overdueCount: 8,
          overdueAmount: 4_752,
          oldestDueDate: "2026-06-15",
        }
      : null,
  );
  const passwordRef = useRef<HTMLInputElement>(null);
  const titleRef = useRef<HTMLSelectElement>(null);
  const membersTableRef = useRef<HTMLDivElement>(null);
  const membersScrollPosition = useRef(0);
  const startupUpdateCheckStarted = useRef(false);

  async function checkForUpdates(manual = false) {
    if (!isTauri()) {
      setUpdaterStatus("Dostupné pouze v nainstalované aplikaci");
      if (manual) setUpdateMessage("Kontrola aktualizací je dostupná v nainstalované aplikaci.");
      return;
    }
    setUpdateChecking(true);
    setUpdaterStatus("Kontroluji aktualizace…");
    setUpdateMessage("");
    try {
      const update = await check({ timeout: 15_000 });
      const checkedAt = new Intl.DateTimeFormat("cs-CZ", {
        dateStyle: "medium",
        timeStyle: "short",
      }).format(new Date());
      localStorage.setItem(LAST_UPDATE_CHECK_KEY, checkedAt);
      setLastUpdateCheck(checkedAt);
      if (update) {
        console.info("Updater: nalezena nová verze");
        setAvailableUpdate(update);
        setUpdaterStatus(`Dostupná verze ${update.version}`);
      } else {
        console.info("Updater: není novější verze");
        setUpdaterStatus("Aplikace je aktuální");
        if (manual) setUpdateMessage("Používáte aktuální verzi aplikace.");
      }
    } catch (reason) {
      console.error("Updater: chyba updateru");
      const errorText = String(reason).toLowerCase();
      const releaseNotFound = errorText.includes("404") || errorText.includes("not found");
      if (releaseNotFound) {
        const message = "Nebyla nalezena žádná publikovaná verze.";
        setUpdaterStatus(message);
        setUpdateMessage(message);
      } else {
        setUpdaterStatus("Kontrola se nezdařila");
      }
      if (manual && !releaseNotFound) {
        setUpdateMessage("Aktualizace se nyní nepodařilo ověřit. Aplikaci můžete dál používat.");
      }
    } finally {
      setUpdateChecking(false);
    }
  }

  async function installAvailableUpdate() {
    if (!availableUpdate) return;
    setUpdateInstalling(true);
    setUpdaterStatus("Stahuji aktualizaci…");
    setUpdateMessage("");
    setUpdateProgress(0);
    let downloaded = 0;
    let total: number | undefined;
    try {
      await availableUpdate.downloadAndInstall((event) => {
        if (event.event === "Started") {
          console.info("Updater: stažení zahájeno");
          total = event.data.contentLength;
        } else if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          if (total) setUpdateProgress(Math.min(100, Math.round((downloaded / total) * 100)));
        } else {
          setUpdateProgress(100);
        }
      });
      console.info("Updater: instalace dokončena");
      setUpdaterStatus("Instalace dokončena");
      await relaunch();
    } catch {
      console.error("Updater: chyba updateru");
      setUpdaterStatus("Instalace se nezdařila");
      setUpdateMessage("Aktualizaci se nepodařilo dokončit. Aplikaci můžete dál používat.");
      setUpdateInstalling(false);
    }
  }

  function changeUpdateCheckEnabled(enabled: boolean) {
    localStorage.setItem(UPDATE_CHECK_ENABLED_KEY, String(enabled));
    setUpdateCheckEnabled(enabled);
  }

  async function createDatabaseBackup() {
    setBackupBusy(true);
    try {
      const backup = await invoke<BackupInfo | null>("create_database_backup");
      if (backup) {
        window.alert(`Záloha byla úspěšně vytvořena.\n\n${backup.fileName}`);
        if (screen === "Správa záloh") await loadDatabaseBackups();
      }
    } catch (message) {
      window.alert(String(message));
    } finally {
      setBackupBusy(false);
    }
  }

  async function restoreDatabaseBackup(path?: string) {
    setBackupBusy(true);
    try {
      const backup = path
        ? backups.find((item) => item.path === path) ?? null
        : await invoke<BackupInfo | null>("select_database_backup");
      if (!backup) return;
      const confirmed = window.confirm(
        `Obnovit databázi z této zálohy?\n\n` +
        `Verze aplikace: ${backup.applicationVersion}\n` +
        `Datum: ${displayBackupDate(backup.createdAt)}\n` +
        `Počet členů: ${backup.memberCount.toLocaleString("cs-CZ")}\n` +
        `Velikost: ${displayFileSize(backup.databaseSize)}\n\n` +
        `Před obnovou bude automaticky vytvořena nouzová záloha.`,
      );
      if (!confirmed) return;
      await invoke("restore_database_backup", { path: backup.path });
      window.alert("Obnova byla dokončena. Aplikace bude nyní restartována.");
      await relaunch();
    } catch (message) {
      window.alert(String(message));
    } finally {
      setBackupBusy(false);
    }
  }

  async function loadDatabaseBackups() {
    setBackupsLoading(true);
    try {
      setBackups(await invoke<BackupInfo[]>("list_database_backups"));
    } catch (message) {
      setError(String(message));
    } finally {
      setBackupsLoading(false);
    }
  }

  const shellUpdater = {
    updater: {
      available: availableUpdate,
      installing: updateInstalling,
      progress: updateProgress,
      message: updateMessage,
      onInstall: () => void installAvailableUpdate(),
      onLater: () => {
        availableUpdate?.close().catch(() => undefined);
        setAvailableUpdate(null);
        setUpdateMessage("");
        setUpdateProgress(null);
      },
    },
    backupBusy,
    onCreateBackup: () => void createDatabaseBackup(),
    onRestoreBackup: () => void restoreDatabaseBackup(),
  };

  useEffect(() => {
    if (screen === "Vstup") {
      passwordRef.current?.focus();
    }
    if (screen === "Pojištěnci") {
      titleRef.current?.focus();
    }
  }, [screen]);

  useEffect(() => {
    if (preview) return;
    invoke<{ initialized: boolean }>("get_auth_status")
      .then((status) => setAuthInitialized(status.initialized))
      .catch((message) => setError(String(message)));
  }, []);

  useEffect(() => {
    if (!user || preview || !updateCheckEnabled || startupUpdateCheckStarted.current) return;
    startupUpdateCheckStarted.current = true;
    void checkForUpdates();
  }, [user, updateCheckEnabled]);

  useEffect(() => {
    if (screen === "Správa záloh" && !preview) void loadDatabaseBackups();
    if (screen === "Doklady o zaplacení" && !preview) void loadReceipts(undefined, "");
    if (screen === "Pojistné události" && !preview) void loadClaimsOverview();
  }, [screen]);

  useEffect(() => {
    if (screen !== "Pojištěnci") return;
    invoke<FormOptions>("get_form_options", {
      affiliation: form.affiliation,
      year: form.registrationYear,
    })
      .then((result) => {
        setOptions(result);
        setForm((current) => ({
          ...current,
          organization: result.organizations.includes(current.organization)
            ? current.organization
            : "",
        }));
      })
      .catch((message) => setError(String(message)));
  }, [screen, form.affiliation, form.registrationYear]);

  useEffect(() => {
    if (screen !== "Pojištěnci") return;
    if (!form.insuranceFrom) {
      setTariff({ premium: 0, months: 0, insuredAmount: 0 });
      return;
    }
    invoke<TariffResult>("calculate_tariff", {
      category: form.category,
      loss: form.loss,
      annualAmount: form.annualAmount,
      insuranceFrom: optional(form.insuranceFrom),
      insuranceTo: optional(form.insuranceTo),
    })
      .then(setTariff)
      .catch((message) => setError(String(message)));
  }, [
    screen,
    form.category,
    form.loss,
    form.annualAmount,
    form.insuranceFrom,
    form.insuranceTo,
  ]);

  useEffect(() => {
    if (screen !== "Přehled" && screen !== "O programu") return;
    if (preview) return;
    invoke<DashboardInfo>("get_dashboard")
      .then(setDashboard)
      .catch((message) => setError(String(message)));
  }, [screen]);

  const registrationNumber = options.lastRegistrationNumber + 1;
  function update<K extends keyof InsuredForm>(key: K, value: InsuredForm[K]) {
    setForm((current) => ({ ...current, [key]: value }));
  }

  async function enter(event: FormEvent) {
    event.preventDefault();
    setError("");
    try {
      if (authInitialized === null) return;
      if (!authInitialized && password !== passwordConfirmation) {
        setError("Zadaná hesla se neshodují.");
        return;
      }
      const result = await invoke<{ user: string; role: string }>(
        authInitialized ? "login" : "initialize_admin",
        { password },
      );
      setUser(result.user);
      setRole(result.role);
      setPassword("");
      setPasswordConfirmation("");
      setAuthInitialized(true);
      setScreen("Přehled");
    } catch (message) {
      setError(String(message));
      passwordRef.current?.focus();
      passwordRef.current?.select();
    }
  }

  async function leaveToLogin() {
    await invoke("logout");
    setUser("");
    setRole("");
    setForm(emptyForm(0));
    setScreen("Vstup");
  }

  function navigate(next: Exclude<Screen, "Vstup">) {
    setError("");
    setNotice("");
    setSelectedMember(null);
    setAgendaMembers([]);
    setAgendaSearch("");
    setMemberPaymentForm(null);
    setPaymentDocumentBasis(null);
    if (next !== "Nastavení") {
      setSettingsSection(null);
      setTariffForm(null);
    }
    if (next === "Pojištěnci") {
      openInsured();
      return;
    }
    if (next === "Seznam") {
      void openMembers();
      return;
    }
    if (next === "Archiv") {
      void openArchive();
      return;
    }
    setScreen(next);
  }

  async function searchAgendaMembers(event?: FormEvent) {
    event?.preventDefault();
    setMembersLoading(true);
    setError("");
    try {
      const result = await invoke<MemberPage>("list_members", {
        search: agendaSearch.trim() || null,
        page: 1,
        pageSize: 25,
        filters: emptyFilters,
      });
      setAgendaMembers(result.members);
    } catch (message) {
      setError(String(message));
    } finally {
      setMembersLoading(false);
    }
  }

  async function selectAgendaMember(member: Member, purpose: "payment" | "receipt") {
    setError("");
    try {
      const current = await invoke<Member>("get_current_member", { rowId: member.rowId });
      setSelectedMember(current);
      if (purpose === "payment") {
        setMemberPayments(await invoke<MemberPayment[]>("list_member_payments", { rowId: current.rowId }));
        newMemberPayment(current);
      } else {
        await loadMemberReceiptData(current);
      }
    } catch (message) {
      setError(String(message));
    }
  }

  async function loadClaimsOverview() {
    setClaimsLoading(true);
    setError("");
    try {
      setAgendaClaims(await invoke<ClaimOverview[]>("list_claims"));
    } catch (message) {
      setError(String(message));
    } finally {
      setClaimsLoading(false);
    }
  }

  function openAgendaClaim(member: Member) {
    setSelectedMember(member);
    setReturnToMember(false);
    setCreatedClaimId(null);
    setEditingClaimId(null);
    setClaimForm(emptyClaimForm(member.rowId));
    setScreen("Nová pojistná událost");
  }

  async function editAgendaClaim(claim: ClaimOverview) {
    setClaimsLoading(true);
    setError("");
    try {
      const member = await invoke<Member>("get_current_member", { rowId: claim.memberRowId });
      const claims = await invoke<Claim[]>("list_member_claims", { rowId: claim.memberRowId });
      const detail = claims.find((item) => item.id === claim.id);
      if (!detail) throw new Error("Pojistná událost nebyla nalezena.");
      setSelectedMember(member);
      setReturnToMember(false);
      setEditingClaimId(claim.id);
      setCreatedClaimId(null);
      setClaimForm({
        insuranceRowId: member.rowId,
        phone: detail.phone ?? "", employer: detail.employer ?? "", occupation: detail.occupation ?? "",
        occurredOn: detail.occurredOn ?? "", reportedOn: detail.reportedOn ?? "",
        assessedDamage: detail.assessedDamage == null ? "" : String(detail.assessedDamage),
        insuranceBenefit: detail.insuranceBenefit == null ? "" : String(detail.insuranceBenefit),
        description: detail.description ?? "", note: detail.note ?? "",
        additionalInformation: detail.additionalInformation ?? "", closedOn: detail.closedOn ?? "",
        handledBy: detail.handledBy ?? "", reportPosition: detail.reportPosition ?? "",
      });
      setScreen("Nová pojistná událost");
    } catch (message) {
      setError(String(message));
    } finally {
      setClaimsLoading(false);
    }
  }

  async function selectPaymentMember(rowId: number) {
    setPaymentLoading(true);
    setError("");
    setNotice("");
    try {
      const draft = await invoke<PaymentOrderDraft>("prepare_payment_order", { rowId });
      setPaymentDraft(draft);
      setLastPaymentPdf("");
    } catch (message) {
      setError(String(message));
    } finally {
      setPaymentLoading(false);
    }
  }

  async function openPaymentForMember(member: Member) {
    setReturnToMember(true);
    setScreen("Příkaz k úhradě");
    await selectPaymentMember(member.rowId);
  }

  function openClaimForMember(member: Member) {
    setReturnToMember(true);
    setCreatedClaimId(null);
    setEditingClaimId(null);
    setClaimForm(emptyClaimForm(member.rowId));
    setScreen("Nová pojistná událost");
  }

  function returnToMemberDetail() {
    setScreen(returnToMember ? "Seznam" : "Pojistné události");
    setReturnToMember(false);
    setClaimForm(null);
    setPaymentDraft(null);
    setCreatedClaimId(null);
    setEditingClaimId(null);
  }

  async function loadMemberClaims(member: Member) {
    setClaimsLoading(true);
    setError("");
    try {
      setMemberClaims(await invoke<Claim[]>("list_member_claims", { rowId: member.rowId }));
    } catch (message) {
      setError(String(message));
    } finally {
      setClaimsLoading(false);
    }
  }

  async function reloadMemberPayments(rowId: number) {
    const [member, payments, auditHistory] = await Promise.all([
      invoke<Member>("get_current_member", { rowId }),
      invoke<MemberPayment[]>("list_member_payments", { rowId }),
      invoke<AuditEntry[]>("get_member_audit_history", { rowId }),
    ]);
    setSelectedMember(member);
    setMemberPayments(payments);
    setMemberAuditHistory(auditHistory);
  }

  async function loadReceipts(memberRowId?: number, search = receiptSearch) {
    setError("");
    try {
      const loaded = await invoke<Receipt[]>("list_receipts", { memberRowId: memberRowId ?? null, search: search || null });
      if (memberRowId) setMemberReceipts(loaded); else setReceipts(loaded);
    } catch (message) {
      setError(String(message));
    }
  }

  async function loadMemberReceiptData(member: Member) {
    setError("");
    try {
      const [loaded, basis] = await Promise.all([
        invoke<Receipt[]>("list_receipts", { memberRowId: member.rowId, search: null }),
        invoke<PaymentDocumentBasis>("get_payment_document_basis", { rowId: member.rowId }),
      ]);
      setMemberReceipts(loaded);
      setPaymentDocumentBasis(basis);
    } catch {
      setPaymentDocumentBasis(null);
      setError("Podklady dokladu se nepodařilo načíst.");
    }
  }

  async function createMemberReceipt(member: Member) {
    setSaving(true);
    setError("");
    try {
      const id = await invoke<number | null>("create_receipt", { rowId: member.rowId });
      await loadMemberReceiptData(member);
      setNotice(id ? "Doklad je připraven." : "Doklad lze vytvořit až po úplné úhradě pojistného.");
    } catch (message) {
      setError(String(message));
    } finally {
      setSaving(false);
    }
  }

  async function receiptAction(command: "open_receipt_pdf" | "export_receipt_pdf" | "send_receipt_email", receipt: Receipt, print = false) {
    setError("");
    try {
      await invoke(command, command === "open_receipt_pdf" ? { id: receipt.id, print } : { id: receipt.id });
      if (command === "send_receipt_email") {
        setNotice("Doklad byl odeslán e-mailem.");
        await loadReceipts(selectedMember?.rowId, selectedMember ? "" : receiptSearch);
      }
    } catch (message) {
      setError(String(message));
    }
  }

  function newMemberPayment(member: Member) {
    setMemberPaymentForm({ insuranceRowId: member.rowId, receivedOn: new Date().toISOString().slice(0, 10), amount: "", method: "Bankovní převod", note: "" });
  }

  function editMemberPayment(member: Member, payment: MemberPayment) {
    setMemberPaymentForm({ id: payment.id, insuranceRowId: member.rowId, receivedOn: payment.receivedOn, amount: String(payment.amount), method: payment.method, note: payment.note ?? "" });
  }

  async function saveMemberPayment() {
    if (!memberPaymentForm || !selectedMember) return;
    setSaving(true);
    setError("");
    try {
      await invoke("save_member_payment", { payment: { ...memberPaymentForm, amount: Number(memberPaymentForm.amount), note: optional(memberPaymentForm.note) } });
      await reloadMemberPayments(selectedMember.rowId);
      await loadReceipts(selectedMember.rowId, "");
      setDashboard(await invoke<DashboardInfo>("get_dashboard"));
      setMemberPaymentForm(null);
      setNotice(memberPaymentForm.id ? "Platba byla upravena." : "Platba byla přidána.");
    } catch (message) {
      setError(String(message));
    } finally {
      setSaving(false);
    }
  }

  async function removeMemberPayment(payment: MemberPayment) {
    if (!selectedMember || !window.confirm(`Opravdu odstranit platbu ${displayCurrency(payment.amount)}?`)) return;
    setSaving(true);
    setError("");
    try {
      await invoke("delete_member_payment", { rowId: selectedMember.rowId, paymentId: payment.id });
      await reloadMemberPayments(selectedMember.rowId);
      setDashboard(await invoke<DashboardInfo>("get_dashboard"));
      setMemberPaymentForm(null);
      setNotice("Platba byla odstraněna.");
    } catch (message) {
      setError(String(message));
    } finally {
      setSaving(false);
    }
  }

  async function saveClaim() {
    if (!claimForm) return;
    setClaimsLoading(true);
    setError("");
    try {
      const payload = {
        ...claimForm,
        assessedDamage: claimForm.assessedDamage ? Number(claimForm.assessedDamage) : null,
        insuranceBenefit: claimForm.insuranceBenefit ? Number(claimForm.insuranceBenefit) : null,
      };
      const id = editingClaimId ?? await invoke<number>("create_claim", {
        claim: {
          ...payload,
        },
      });
      if (editingClaimId) await invoke("update_claim", { id: editingClaimId, claim: payload });
      setCreatedClaimId(id);
      setNotice(editingClaimId ? "Pojistná událost byla upravena." : "Pojistná událost byla vytvořena.");
      setEditingClaimId(null);
      if (selectedMember) await loadMemberClaims(selectedMember);
      await loadClaimsOverview();
    } catch (message) {
      setError(String(message));
    } finally {
      setClaimsLoading(false);
    }
  }

  async function createPaymentPdf() {
    if (!paymentDraft) return;
    setPaymentLoading(true);
    setError("");
    try {
      const saved = await invoke<string | null>("generate_payment_order_pdf", {
        order: { rowId: paymentDraft.rowId },
      });
      if (saved) setLastPaymentPdf(saved);
      setNotice(saved ? `PDF bylo uloženo do: ${saved}` : "Uložení PDF bylo zrušeno.");
    } catch (message) {
      setError(String(message));
    } finally {
      setPaymentLoading(false);
    }
  }

  async function openPaymentPdf(folder: boolean) {
    if (!lastPaymentPdf) return;
    try {
      await invoke("open_generated_pdf", { path: lastPaymentPdf, folder });
    } catch (message) {
      setError(String(message));
    }
  }

  async function printPaymentOrder() {
    if (!paymentDraft) return;
    setPaymentLoading(true);
    setError("");
    try {
      await invoke("audit_payment_order_print", { rowId: paymentDraft.rowId });
      window.print();
    } catch (message) {
      setError(String(message));
    } finally {
      setPaymentLoading(false);
    }
  }

  async function openPaymentSettings() {
    setSettingsSection("payments");
    setError("");
    try {
      setPaymentSettings(await invoke<PaymentSettings>("get_payment_settings"));
    } catch (message) {
      setError(String(message));
    }
  }

  async function savePaymentSettings() {
    if (!paymentSettings) return;
    setSaving(true);
    setError("");
    try {
      await invoke("save_payment_settings", { settings: paymentSettings });
      setNotice("Platební údaje byly uloženy.");
    } catch (message) {
      setError(String(message));
    } finally {
      setSaving(false);
    }
  }

  async function openEmailSettings() {
    setSettingsSection("email"); setError("");
    try { setEmailSettings({ ...(await invoke<EmailSettings>("get_email_settings")), password: "" }); } catch (message) { setError(String(message)); }
  }

  async function saveEmailSettings() {
    if (!emailSettings) return; setSaving(true); setError("");
    try { await invoke("save_email_settings", { settings: { ...emailSettings, password: emailSettings.password || null } }); setNotice("Nastavení SMTP bylo uloženo. Heslo je ve Windows Credential Manageru."); await openEmailSettings(); } catch (message) { setError(String(message)); } finally { setSaving(false); }
  }

  async function openReceiptSettings() {
    setSettingsSection("receipts"); setError("");
    try { setReceiptSettings(await invoke<ReceiptSettings>("get_receipt_settings")); } catch (message) { setError(String(message)); }
  }

  async function saveReceiptSettings() {
    if (!receiptSettings) return; setSaving(true); setError("");
    try { await invoke("save_receipt_settings", { settings: receiptSettings }); setNotice("Nastavení dokladů bylo uloženo."); } catch (message) { setError(String(message)); } finally { setSaving(false); }
  }

  async function openTariffSettings() {
    setSettingsSection("tariffs");
    setTariffForm(null);
    setTariffsLoading(true);
    setError("");
    try {
      setTariffRates(await invoke<TariffRate[]>("list_tariff_rates"));
    } catch (message) {
      setError(String(message));
    } finally {
      setTariffsLoading(false);
    }
  }

  async function saveTariffRate() {
    if (!tariffForm) return;
    setTariffsLoading(true);
    setError("");
    try {
      await invoke<number>("save_tariff_rate", { rate: tariffForm });
      setTariffRates(await invoke<TariffRate[]>("list_tariff_rates"));
      setTariffForm(null);
      setNotice("Sazba pojistného byla uložena.");
    } catch (message) {
      setError(String(message));
    } finally {
      setTariffsLoading(false);
    }
  }

  async function toggleTariffRate(rate: TariffRate) {
    setTariffsLoading(true);
    setError("");
    try {
      await invoke<number>("save_tariff_rate", {
        rate: { ...rate, active: !rate.active },
      });
      setTariffRates(await invoke<TariffRate[]>("list_tariff_rates"));
    } catch (message) {
      setError(String(message));
    } finally {
      setTariffsLoading(false);
    }
  }

  function openInsured() {
    setError("");
    setNotice("");
    const activeYear = dashboard?.activeInsuranceYear;
    if (!activeYear) {
      setError("Aktivní pojistný rok se nepodařilo načíst.");
      return;
    }
    setForm(emptyForm(activeYear));
    setScreen("Pojištěnci");
  }

  async function loadMembers(page = 1, search = activeSearch, filters = memberFilters) {
    setMembersLoading(true);
    setError("");
    try {
      const result = await invoke<MemberPage>("list_members", {
        search: search || null,
        page,
        pageSize: 50,
        filters,
      });
      setMemberPage(result);
    } catch (message) {
      setError(String(message));
    } finally {
      setMembersLoading(false);
    }
  }

  async function openMembers() {
    setError("");
    setNotice("");
    setMemberSearch("");
    setActiveSearch("");
    setMemberFilters(emptyFilters);
    setScreen("Seznam");
    await loadMembers(1, "", emptyFilters);
  }

  async function openOverdueMembers() {
    const filters = { ...emptyFilters, overdue: "po_splatnosti" };
    setError("");
    setNotice("");
    setMemberSearch("");
    setActiveSearch("");
    setMemberFilters(filters);
    setScreen("Seznam");
    await loadMembers(1, "", filters);
  }

  async function searchMembers(event: FormEvent) {
    event.preventDefault();
    const search = memberSearch.trim();
    setActiveSearch(search);
    await loadMembers(1, search);
  }

  async function openMember(rowId: number) {
    setError("");
    membersScrollPosition.current = membersTableRef.current?.scrollTop ?? 0;
    try {
      const [member, history, auditHistory, payments] = await Promise.all([
        invoke<Member>("get_current_member", { rowId }),
        invoke<Member[]>("get_member_history", { rowId }),
        invoke<AuditEntry[]>("get_member_audit_history", { rowId }),
        invoke<MemberPayment[]>("list_member_payments", { rowId }),
      ]);
      setSelectedMember(member);
      setMemberHistory(history);
      setMemberAuditHistory(auditHistory);
      setMemberPayments(payments);
      setMemberPaymentForm(null);
      setHistoryMember(null);
      setDetailTab("overview");
      setEditingMember(false);
    } catch (message) {
      setError(String(message));
    }
  }

  useEffect(() => {
    if (screen !== "Seznam" || !selectedMember) return;
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !editingMember) {
        event.preventDefault();
        closeMemberDetail();
      }
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s" && editingMember) {
        event.preventDefault();
        void saveMemberEdit();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [screen, selectedMember, editingMember, memberEdit]);

  function closeMemberDetail() {
      setSelectedMember(null);
      setHistoryMember(null);
      setEditingMember(false);
    window.requestAnimationFrame(() => {
      if (membersTableRef.current) {
        membersTableRef.current.scrollTop = membersScrollPosition.current;
      }
    });
  }

  function startMemberEdit(member: Member) {
    setMemberEdit({
      rowId: member.rowId,
      title: member.title ?? "",
      lastName: member.lastName ?? "",
      firstName: member.firstName ?? "",
      personalId: member.personalId ?? "",
      registrationNumber: member.registrationNumber ? Number(member.registrationNumber) : null,
      city: member.city ?? "",
      address: member.address ?? "",
      postalCode: member.postalCode ?? "",
      country: member.country ?? "",
      organization: member.organization ?? "",
      affiliation: member.affiliation ?? "",
      code: member.code ?? "",
      email: member.email ?? "",
      note: member.note ?? "",
      actualPayment: member.actualPayment ? Number(member.actualPayment) : 0,
      actualTermination: member.actualTermination?.slice(0, 10) ?? "",
    });
    setEditingMember(true);
  }

  async function saveMemberEdit() {
    if (!memberEdit) return;
    setSaving(true);
    setError("");
    try {
      const updated = await invoke<Member>("update_current_member", {
        member: {
          ...memberEdit,
          actualTermination: optional(memberEdit.actualTermination),
        },
      });
      setSelectedMember(updated);
      setMemberHistory(await invoke<Member[]>("get_member_history", { rowId: updated.rowId }));
      setEditingMember(false);
      setMemberEdit(null);
      setNotice("Údaje člena byly uloženy.");
    } catch (message) {
      setError(String(message));
    } finally {
      setSaving(false);
    }
  }

  async function openArchive() {
    setError("");
    setNotice("");
    setArchiveYear(null);
    setSelectedArchiveMember(null);
    setArchiveSearch("");
    setActiveArchiveSearch("");
    setScreen("Archiv");
    setArchiveLoading(true);
    try {
      setArchiveYears(await invoke<ArchiveYear[]>("list_archive_years"));
    } catch (message) {
      setError(String(message));
    } finally {
      setArchiveLoading(false);
    }
  }

  async function loadArchiveMembers(
    year: number,
    page = 1,
    search = activeArchiveSearch,
  ) {
    setArchiveLoading(true);
    setError("");
    try {
      setArchivePage(
        await invoke<MemberPage>("list_archive_members", {
          year,
          search: search || null,
          page,
          pageSize: 50,
          filters: archiveFilters,
        }),
      );
    } catch (message) {
      setError(String(message));
    } finally {
      setArchiveLoading(false);
    }
  }

  async function selectArchiveYear(year: number) {
    setArchiveYear(year);
    setArchiveSearch("");
    setActiveArchiveSearch("");
    setArchiveFilters(emptyFilters);
    await loadArchiveMembers(year, 1, "");
  }

  async function searchArchive(event: FormEvent) {
    event.preventDefault();
    if (archiveYear === null) return;
    const search = archiveSearch.trim();
    setActiveArchiveSearch(search);
    await loadArchiveMembers(archiveYear, 1, search);
  }

  async function openArchiveMember(rowId: number) {
    setError("");
    try {
      setSelectedArchiveMember(await invoke<Member>("get_member", { rowId }));
    } catch (message) {
      setError(String(message));
    }
  }

  function cancelInsured() {
    setError("");
    setNotice("");
    setForm(emptyForm(dashboard?.activeInsuranceYear ?? 0));
    setScreen("Přehled");
  }

  function showLastRegistration() {
    setNotice(
      `Poslední zavedené evidenční číslo je ${options.lastRegistrationNumber}${options.lastClient}`,
    );
  }

  async function save(closeAfterSave: boolean) {
    setError("");
    setNotice("");
    setSaving(true);
    try {
      const result = await invoke<{ identifier: number; registrationNumber: number }>(
        "save_insured",
        {
          insured: {
            ...form,
            title: optional(form.title),
            lastName: optional(form.lastName),
            firstName: optional(form.firstName),
            personalId: optional(form.personalId),
            organization: optional(form.organization),
            city: optional(form.city),
            address: optional(form.address),
            postalCode: optional(form.postalCode),
            country: optional(form.country),
            note: optional(form.note),
            insuranceFrom: optional(form.insuranceFrom),
            insuranceTo: optional(form.insuranceTo),
            actualPayment: form.actualPayment === "" ? null : Number(form.actualPayment),
            email: optional(form.email),
          },
        },
      );
      setNotice(`Záznam byl uložen. Evidenční číslo: ${result.registrationNumber}.`);
      if (closeAfterSave) {
        setForm(emptyForm(dashboard?.activeInsuranceYear ?? 0));
        setScreen("Přehled");
      } else {
        const activeYear = dashboard?.activeInsuranceYear;
        if (!activeYear) {
          throw new Error("Aktivní pojistný rok se nepodařilo načíst.");
        }
        setForm(emptyForm(activeYear));
        const refreshed = await invoke<FormOptions>("get_form_options", {
          affiliation: "FVČ",
          year: activeYear,
        });
        setOptions(refreshed);
        titleRef.current?.focus();
      }
    } catch (message) {
      setError(String(message));
    } finally {
      setSaving(false);
    }
  }

  if (screen === "Vstup") {
    return (
      <main className="login-screen">
        <form className="login-card" onSubmit={enter}>
          <div className="app-mark">
            <ShieldCheck />
          </div>
          <h1>Pojištění</h1>
          <p>{authInitialized === false ? "Vytvoření účtu správce" : "Vstup do databáze"}</p>
          {error && <div className="message error">{error}</div>}
          {authInitialized === false && <div className="message info">Při prvním spuštění nastavte heslo správce o délce alespoň 12 znaků.</div>}
          <label>
            {authInitialized === false ? "Nové heslo" : "Heslo"}
            <input
              ref={passwordRef}
              type="password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              minLength={authInitialized === false ? 12 : undefined}
              autoComplete={authInitialized === false ? "new-password" : "current-password"}
            />
          </label>
          {authInitialized === false && (
            <label>
              Potvrzení hesla
              <input
                type="password"
                value={passwordConfirmation}
                onChange={(event) => setPasswordConfirmation(event.target.value)}
                minLength={12}
                autoComplete="new-password"
              />
            </label>
          )}
          <div className="login-actions">
            <button type="submit" className="primary" disabled={authInitialized === null}>
              <LogIn /> {authInitialized === false ? "Vytvořit účet" : "Vstup"}
            </button>
            <button type="button" onClick={() => invoke("quit_application")}>
              Zavřít
            </button>
          </div>
        </form>
      </main>
    );
  }

  if (screen === "Přehled") {
    return (
      <Shell {...shellUpdater} active="Přehled" user={user} onNavigate={navigate} onLogout={leaveToLogin}>
        <div className="page dashboard-page">
          <header className="page-header">
            <div>
              <small>Pojištění</small>
              <h1>Přehled</h1>
            </div>
          </header>
          {error && <div className="message error">{error}</div>}
          {notice && <div className="message success">{notice}</div>}
          <section className="dashboard-grid">
            <article><Archive /><small>Aktivní pojistný rok</small><strong>{dashboard?.activeInsuranceYear ?? "—"}</strong></article>
            <article><Users /><small>Počet pojištěnců</small><strong>{dashboard ? new Intl.NumberFormat("cs-CZ").format(dashboard.memberCount) : "—"}</strong></article>
            <article><Save /><small>Poslední evidenční číslo</small><strong>{dashboard?.lastRegistrationNumber ?? "—"}</strong></article>
            <article><Database /><small>Datum databáze</small><strong>{displayDate(dashboard?.databaseDate)}</strong></article>
            <article><Info /><small>Verze programu</small><strong>{dashboard?.programVersion ?? "—"}</strong></article>
            <article className="dashboard-card-action" role="button" tabIndex={0} onClick={() => void openOverdueMembers()} onKeyDown={(event) => { if (event.key === "Enter" || event.key === " ") void openOverdueMembers(); }}>
              <TriangleAlert />
              <small>Pojistky po splatnosti</small>
              <strong>{dashboard ? new Intl.NumberFormat("cs-CZ").format(dashboard.overdueCount) : "—"}</strong>
              <span>Neuhrazeno: {dashboard ? displayCurrency(dashboard.overdueAmount) : "—"}</span>
              <span>Nejstarší splatnost: {displayDate(dashboard?.oldestDueDate)}</span>
            </article>
          </section>
        </div>
      </Shell>
    );
  }

  if (screen === "Přidat platbu") {
    return (
      <Shell {...shellUpdater} active="Přidat platbu" user={user} onNavigate={navigate} onLogout={leaveToLogin}>
        <div className="page agenda-page">
          <header className="page-header"><div><small>Platby</small><h1>Přidat platbu</h1></div></header>
          {error && <div className="message error">{error}</div>}
          {notice && <div className="message success">{notice}</div>}
          <form className="search-bar" onSubmit={searchAgendaMembers}>
            <Search /><input value={agendaSearch} onChange={(event) => setAgendaSearch(event.target.value)} placeholder="Jméno, evidenční číslo, kód OC nebo rodné číslo" />
            <button className="primary">Vyhledat člena</button>
          </form>
          {agendaMembers.length > 0 && <div className="agenda-member-results"><table><thead><tr><th>Evidenční číslo</th><th>Člen</th><th>Kód OC</th><th>Rok</th><th>Stav úhrady</th><th></th></tr></thead><tbody>
            {agendaMembers.map((member) => <tr key={member.rowId}><td>{display(member.registrationNumber)}</td><td>{member.insured}</td><td>{display(member.code)}</td><td>{insuranceYear(member)}</td><td>{paymentSummary(member).label}</td><td><button onClick={() => void selectAgendaMember(member, "payment")}>Vybrat</button></td></tr>)}
          </tbody></table></div>}
          {selectedMember && <section className="agenda-workspace">
            <MemberHeading member={selectedMember} />
            <section className="payment-summary-cards">
              <div><span>Roční pojistné</span><strong>{displayCurrency(selectedMember.premium)}</strong></div>
              <div><span>Skutečně uhrazeno</span><strong>{displayCurrency(selectedMember.actualPayment)}</strong></div>
              <div><span>Zbývá uhradit</span><strong>{displayCurrency(Math.max(Number(selectedMember.premium ?? 0) - Number(selectedMember.actualPayment ?? 0), 0))}</strong></div>
              <div><span>Stav</span><strong>{paymentSummary(selectedMember).label}</strong></div>
            </section>
            {memberPaymentForm && <section className="member-payment-form">
              <h3>Nová platba</h3>
              <label>Člen<input value={selectedMember.insured} readOnly /></label>
              <label>Evidenční číslo<input value={selectedMember.registrationNumber ?? ""} readOnly /></label>
              <label>Pojistný rok<input value={insuranceYear(selectedMember)} readOnly /></label>
              <label>Datum úhrady<input type="date" value={memberPaymentForm.receivedOn} onChange={(event) => setMemberPaymentForm({ ...memberPaymentForm, receivedOn: event.target.value })} /></label>
              <label>Částka (Kč)<input type="number" min="1" value={memberPaymentForm.amount} onChange={(event) => setMemberPaymentForm({ ...memberPaymentForm, amount: event.target.value })} /></label>
              <label>Způsob úhrady<select value={memberPaymentForm.method} onChange={(event) => setMemberPaymentForm({ ...memberPaymentForm, method: event.target.value as MemberPaymentForm["method"] })}><option>Bankovní převod</option><option>Hotově</option><option>Jiné</option></select></label>
              <label className="wide">Poznámka<textarea value={memberPaymentForm.note} onChange={(event) => setMemberPaymentForm({ ...memberPaymentForm, note: event.target.value })} /></label>
              <footer><button className="primary" disabled={saving} onClick={saveMemberPayment}><Save /> Uložit platbu</button></footer>
            </section>}
          </section>}
        </div>
      </Shell>
    );
  }

  if (screen === "Pojistné události") {
    const query = agendaSearch.trim().toLocaleLowerCase("cs-CZ");
    const visibleClaims = agendaClaims.filter((claim) =>
      (!query || `${claim.id} ${claim.memberName} ${claim.registrationNumber} ${claim.description ?? ""}`.toLocaleLowerCase("cs-CZ").includes(query)) &&
      (!claimYearFilter || String(claim.insuranceYear) === claimYearFilter) &&
      (!claimStatusFilter || claim.status === claimStatusFilter) &&
      (!claimOcFilter || claim.organizationCode === claimOcFilter)
    );
    return (
      <Shell {...shellUpdater} active="Pojistné události" user={user} onNavigate={navigate} onLogout={leaveToLogin}>
        <div className="page agenda-page">
          <header className="page-header"><div><small>Evidence</small><h1>Pojistné události</h1></div></header>
          {error && <div className="message error">{error}</div>}
          {notice && <div className="message success">{notice}</div>}
          <form className="search-bar" onSubmit={searchAgendaMembers}><Search /><input value={agendaSearch} onChange={(event) => setAgendaSearch(event.target.value)} placeholder="Člen, číslo události nebo popis" /><button className="primary">Vyhledat člena pro novou událost</button></form>
          {agendaMembers.length > 0 && <div className="agenda-member-results"><table><thead><tr><th>Evidenční číslo</th><th>Člen</th><th>Kód OC</th><th>Rok</th><th></th></tr></thead><tbody>{agendaMembers.map((member) => <tr key={member.rowId}><td>{display(member.registrationNumber)}</td><td>{member.insured}</td><td>{display(member.code)}</td><td>{insuranceYear(member)}</td><td><button className="action-claim" onClick={() => openAgendaClaim(member)}><Plus /> Nová událost</button></td></tr>)}</tbody></table></div>}
          <div className="agenda-filters">
            <input value={claimYearFilter} onChange={(event) => setClaimYearFilter(event.target.value.replace(/\D/g, ""))} placeholder="Rok" />
            <select value={claimStatusFilter} onChange={(event) => setClaimStatusFilter(event.target.value)}><option value="">Stav: vše</option><option>Otevřená</option><option>Uzavřená</option></select>
            <input value={claimOcFilter} onChange={(event) => setClaimOcFilter(event.target.value)} placeholder="Kód OC" />
          </div>
          <div className="claims-table"><table><thead><tr><th>Číslo události</th><th>Člen</th><th>Evidenční číslo</th><th>Kód OC</th><th>Datum vzniku</th><th>Datum nahlášení</th><th>Typ události</th><th>Stav</th><th>Požadovaná částka</th><th>Vyplacená částka</th><th>Poslední změna</th><th>Akce</th></tr></thead><tbody>
            {visibleClaims.map((claim) => <tr key={claim.id}><td>{claim.id}</td><td>{claim.memberName}</td><td>{claim.registrationNumber}</td><td>{claim.organizationCode}</td><td>{displayDate(claim.occurredOn)}</td><td>{displayDate(claim.reportedOn)}</td><td>{display(claim.description)}</td><td>{claim.status}</td><td>{displayCurrency(claim.assessedDamage)}</td><td>{displayCurrency(claim.insuranceBenefit)}</td><td>{displayDateTime(claim.lastChanged)}</td><td className="row-actions"><button title="Upravit událost" onClick={() => void editAgendaClaim(claim)}><Pencil /></button><button title="Otevřít detail člena" onClick={() => { setScreen("Seznam"); void openMember(claim.memberRowId); }}><Users /></button></td></tr>)}
            {!claimsLoading && visibleClaims.length === 0 && <tr><td colSpan={12} className="empty-row">Nebyly nalezeny žádné pojistné události.</td></tr>}
          </tbody></table></div>
        </div>
      </Shell>
    );
  }

  if (screen === "Přehled pro pojišťovnu") {
    return (
      <Shell {...shellUpdater} active="Přehled pro pojišťovnu" user={user} onNavigate={navigate} onLogout={leaveToLogin}>
        <div className="page">
          <header className="page-header">
            <div><small>Pojištění</small><h1>Přehled pro pojišťovnu</h1></div>
          </header>
          <div className="message">Modul se připravuje.</div>
        </div>
      </Shell>
    );
  }

  if (screen === "Nová pojistná událost" && selectedMember && claimForm) {
    const updateClaim = <K extends keyof ClaimForm>(key: K, value: ClaimForm[K]) =>
      setClaimForm((current) => current ? { ...current, [key]: value } : current);
    return (
      <Shell {...shellUpdater} active={returnToMember ? "Seznam" : "Pojistné události"} user={user} onNavigate={navigate} onLogout={leaveToLogin}>
        <div className="page claim-page">
          <header className="page-header">
            <div><small>Pojistné události</small><h1>Nová pojistná událost</h1></div>
          </header>
          {error && <div className="message error">{error}</div>}
          {notice && <div className="message success">{notice}</div>}
          {createdClaimId ? (
            <section className="claim-created">
              <Check />
              <h2>Pojistná událost byla uložena.</h2>
              <p>Číslo pojistné události: <strong>{createdClaimId}</strong></p>
              <div className="form-actions">
                <button className="action-claim" onClick={() => { setCreatedClaimId(null); setClaimForm(emptyClaimForm(selectedMember.rowId)); }}>Založit další událost</button>
                <button className="action-neutral" onClick={returnToMemberDetail}><ArrowLeft /> {returnToMember ? "Zpět na detail člena" : "Zpět na pojistné události"}</button>
              </div>
            </section>
          ) : (
            <>
              <section className="claim-member-summary">
                <div><small>Člen</small><strong>{selectedMember.insured}</strong></div>
                <div><small>Evidenční číslo</small><strong>{display(selectedMember.registrationNumber)}</strong></div>
                <div><small>Bydliště</small><strong>{display(selectedMember.address)}, {display(selectedMember.postalCode)} {display(selectedMember.city)}</strong></div>
                <div><small>Organizace</small><strong>{display(selectedMember.organization)}</strong></div>
                <div><small>Odborová příslušnost</small><strong>{display(selectedMember.affiliation)}</strong></div>
                <div><small>Pojistné období</small><strong>{displayDate(selectedMember.insuranceFrom)} – {displayDate(selectedMember.insuranceTo)}</strong></div>
                <div><small>Pojistná částka</small><strong>{displayCurrency(selectedMember.annualPremium)}</strong></div>
                <div><small>Stav pojištění</small><strong>{selectedMember.actualTermination ? "Ukončené" : "Aktivní"}</strong></div>
              </section>
              <section className="claim-form">
                <label>Telefon<input value={claimForm.phone} onChange={(event) => updateClaim("phone", event.target.value)} /></label>
                <label>Zaměstnavatel<input value={claimForm.employer} onChange={(event) => updateClaim("employer", event.target.value)} /></label>
                <label>Povolání<input value={claimForm.occupation} onChange={(event) => updateClaim("occupation", event.target.value)} /></label>
                <label>Datum vzniku pojistné události<input type="date" value={claimForm.occurredOn} onChange={(event) => updateClaim("occurredOn", event.target.value)} /></label>
                <label>Datum oznámení<input type="date" value={claimForm.reportedOn} onChange={(event) => updateClaim("reportedOn", event.target.value)} /></label>
                <label>Zjištěná škoda (Kč)<input type="number" min="0" value={claimForm.assessedDamage} onChange={(event) => updateClaim("assessedDamage", event.target.value)} /></label>
                <label>Pojistné plnění (Kč)<input type="number" min="0" value={claimForm.insuranceBenefit} onChange={(event) => updateClaim("insuranceBenefit", event.target.value)} /></label>
                <label>Datum ukončení<input type="date" value={claimForm.closedOn} onChange={(event) => updateClaim("closedOn", event.target.value)} /></label>
                <label className="wide">Popis události<textarea value={claimForm.description} onChange={(event) => updateClaim("description", event.target.value)} /></label>
                <label className="wide">Poznámky<textarea value={claimForm.note} onChange={(event) => updateClaim("note", event.target.value)} /></label>
                <label className="wide">Doplňky a informace<textarea value={claimForm.additionalInformation} onChange={(event) => updateClaim("additionalInformation", event.target.value)} /></label>
                <label>Řeší makléř s pojišťovnou<input value={claimForm.handledBy} onChange={(event) => updateClaim("handledBy", event.target.value)} /></label>
                <label>Poloha v sestavě<input value={claimForm.reportPosition} onChange={(event) => updateClaim("reportPosition", event.target.value)} /></label>
                <footer className="form-actions wide">
                  <button className="action-claim" disabled={claimsLoading} onClick={saveClaim}><Plus /> Uložit pojistnou událost</button>
                  <button className="action-neutral" disabled={claimsLoading} onClick={returnToMemberDetail}><ArrowLeft /> {returnToMember ? "Zrušit a vrátit se na člena" : "Zrušit"}</button>
                </footer>
              </section>
            </>
          )}
        </div>
      </Shell>
    );
  }

  if (screen === "Příkaz k úhradě") {
    return (
      <Shell {...shellUpdater} active="Příkaz k úhradě" user={user} onNavigate={navigate} onLogout={leaveToLogin}>
        <div className="page payment-order-page">
          <header className="page-header">
            <div><small>Platby</small><h1>Příkaz k úhradě</h1></div>
            {returnToMember && <button className="action-neutral" onClick={returnToMemberDetail}><ArrowLeft /> Zrušit a vrátit se na člena</button>}
          </header>
          {error && <div className="message error">{error}</div>}
          {notice && <div className="message success">{notice}</div>}
          {paymentDraft && (
            <section className="payment-workspace">
              <header>
                <div><small>Vybraný člen</small><h2>{paymentDraft.payerName}</h2><p>Evidenční číslo {paymentDraft.registrationNumber} · {paymentDraft.organization || "Organizace neuvedena"}</p></div>
                <button onClick={returnToMemberDetail}><ArrowLeft /> Zpět na detail člena</button>
              </header>
              {paymentDraft.validationErrors.map((validationError) => (
                <div className="message warning" key={validationError}><TriangleAlert /> {validationError}</div>
              ))}
              {paymentDraft.amountDue === 0 && (
                <div className="message success"><Check /> Pojištění za rok {paymentDraft.insuranceYear} je již plně uhrazené. Roční pojistné: {displayCurrency(paymentDraft.annualPremium)}, uhrazeno: {displayCurrency(paymentDraft.actualPayment)}, nedoplatek: 0 Kč.</div>
              )}
              <div className="payment-layout">
                <article className="payment-preview">
                  <small>Federace vlakových čet</small>
                  <h2>PŘÍKAZ K ÚHRADĚ</h2>
                  <div className="payment-parties">
                    <div><strong>Plátce</strong><span>{paymentDraft.payerName}</span><span>{display(paymentDraft.address)}</span><span>{paymentDraft.postalCode} {paymentDraft.city}</span></div>
                    <div><strong>Příjemce</strong><span>{display(paymentDraft.recipientName)}</span><span>Číslo účtu: {display(paymentDraft.account)}</span>{paymentDraft.iban && <span>IBAN: {paymentDraft.iban}</span>}{paymentDraft.bic && <span>Banka (BIC/SWIFT): {paymentDraft.bic}</span>}</div>
                  </div>
                  <dl>
                    <dt>Částka</dt><dd>{displayCurrency(paymentDraft.amountDue)}</dd>
                    <dt>Variabilní symbol</dt><dd>{paymentDraft.variableSymbol}</dd>
                    <dt>Datum vytvoření</dt><dd>{displayDate(paymentDraft.issueDate)}</dd>
                    <dt>Pojistný rok</dt><dd>{paymentDraft.insuranceYear}</dd>
                    <dt>Konstantní symbol</dt><dd>{display(paymentDraft.constantSymbol)}</dd>
                    <dt>Splatnost</dt><dd>{displayDate(paymentDraft.dueDate)}</dd>
                    <dt>Zpráva pro příjemce</dt><dd>{display(paymentDraft.message)}</dd>
                  </dl>
                  <div className="qr-placeholder">Prostor pro QR platbu</div>
                </article>
                <aside className="payment-editor">
                  <h3>Kontrola příkazu</h3>
                  <div className="payment-balance">
                    <span>Roční pojistné <strong>{displayCurrency(paymentDraft.annualPremium)}</strong></span>
                    <span>Skutečně uhrazeno <strong>{displayCurrency(paymentDraft.actualPayment)}</strong></span>
                    <span>Nedoplatek <strong>{displayCurrency(paymentDraft.amountDue)}</strong></span>
                  </div>
                  <button className="action-payment" disabled={paymentLoading || paymentDraft.validationErrors.length > 0} onClick={createPaymentPdf}>
                    <FileText /> Vytvořit PDF
                  </button>
                  <button className="action-neutral" disabled={paymentLoading || paymentDraft.validationErrors.length > 0} onClick={printPaymentOrder}><Printer /> Tisk</button>
                  <button className="action-neutral" disabled title="Odesílání e-mailem bude doplněno v budoucím sprintu."><Mail /> Odeslat e-mailem</button>
                  {lastPaymentPdf && <div className="generated-pdf-actions">
                    <button onClick={() => openPaymentPdf(false)}>Otevřít PDF</button>
                    <button onClick={() => openPaymentPdf(true)}>Otevřít složku</button>
                    {returnToMember && <button onClick={returnToMemberDetail}>Zpět na detail člena</button>}
                  </div>}
                  {returnToMember && <button className="action-neutral" onClick={returnToMemberDetail}><ArrowLeft /> Zrušit a vrátit se na člena</button>}
                  <small>Vytvoření PDF nemění údaje člena ani pojistný záznam.</small>
                </aside>
              </div>
            </section>
          )}
        </div>
      </Shell>
    );
  }

  if (screen === "Nastavení") {
    if (settingsSection === "updates") {
      return (
        <Shell {...shellUpdater} active="Nastavení" user={user} onNavigate={navigate} onLogout={leaveToLogin}>
          <div className="page update-settings-page">
            <header className="page-header">
              <div><small>Nastavení</small><h1>Aktualizace</h1></div>
              <button onClick={() => setSettingsSection(null)}><ArrowLeft /> Zpět na nastavení</button>
            </header>
            <section className="update-settings-card">
              <label className="checkbox">
                <input
                  type="checkbox"
                  checked={updateCheckEnabled}
                  onChange={(event) => changeUpdateCheckEnabled(event.target.checked)}
                />
                Kontrolovat aktualizace při spuštění
              </label>
              <p>Kontrola probíhá na pozadí a nebrání spuštění ani používání aplikace.</p>
              <p>Poslední kontrola: <strong>{lastUpdateCheck}</strong></p>
              <button className="primary" disabled={updateChecking} onClick={() => void checkForUpdates(true)}>
                {updateChecking ? "Kontroluji…" : "Vyhledat aktualizace"}
              </button>
              {updateMessage && <div className="message info">{updateMessage}</div>}
            </section>
          </div>
        </Shell>
      );
    }
    if (settingsSection === "payments") {
      return (
        <Shell {...shellUpdater} active="Nastavení" user={user} onNavigate={navigate} onLogout={leaveToLogin}>
          <div className="page payment-settings-page">
            <header className="page-header">
              <div><small>Nastavení</small><h1>Platební údaje</h1></div>
              <button onClick={() => setSettingsSection(null)}><ArrowLeft /> Zpět na nastavení</button>
            </header>
            {error && <div className="message error">{error}</div>}
            {notice && <div className="message success">{notice}</div>}
            {paymentSettings && (
              <section className="payment-settings-form">
                <label>Název příjemce<input value={paymentSettings.recipientName} onChange={(event) => setPaymentSettings({ ...paymentSettings, recipientName: event.target.value })} /></label>
                <label>Číslo účtu<input value={paymentSettings.accountNumber} onChange={(event) => setPaymentSettings({ ...paymentSettings, accountNumber: event.target.value })} /></label>
                <label>Kód banky<input value={paymentSettings.bankCode} onChange={(event) => setPaymentSettings({ ...paymentSettings, bankCode: event.target.value.replace(/\D/g, "") })} /></label>
                <label>IBAN<input value={paymentSettings.iban} onChange={(event) => setPaymentSettings({ ...paymentSettings, iban: event.target.value.toUpperCase().replace(/\s/g, "") })} /></label>
                <label>BIC / SWIFT<input value={paymentSettings.bic} onChange={(event) => setPaymentSettings({ ...paymentSettings, bic: event.target.value.toUpperCase() })} /></label>
                <label>Konstantní symbol<input value={paymentSettings.constantSymbol} onChange={(event) => setPaymentSettings({ ...paymentSettings, constantSymbol: event.target.value.replace(/\D/g, "") })} /></label>
                <label>Výchozí splatnost (dní)<input type="number" min="0" max="365" value={paymentSettings.defaultDueDays} onChange={(event) => setPaymentSettings({ ...paymentSettings, defaultDueDays: Number(event.target.value) })} /></label>
                <label className="wide">Text zprávy pro příjemce<textarea value={paymentSettings.messageTemplate} onChange={(event) => setPaymentSettings({ ...paymentSettings, messageTemplate: event.target.value })} /></label>
                <p className="wide settings-help">Dostupné značky: <code>{"{rok}"}</code> a <code>{"{evidencni_cislo}"}</code>. Účet lze zadat jako české číslo účtu s kódem banky nebo jako IBAN.</p>
                <footer className="form-actions wide"><button className="primary" disabled={saving || role !== "Správce"} onClick={savePaymentSettings}><Save /> Uložit platební údaje</button></footer>
              </section>
            )}
          </div>
        </Shell>
      );
    }
    if (settingsSection === "email") {
      return <Shell {...shellUpdater} active="Nastavení" user={user} onNavigate={navigate} onLogout={leaveToLogin}><div className="page payment-settings-page">
        <header className="page-header"><div><small>Nastavení</small><h1>E-mail (SMTP)</h1></div><button onClick={() => setSettingsSection(null)}><ArrowLeft /> Zpět na nastavení</button></header>
        {error && <div className="message error">{error}</div>}{notice && <div className="message success">{notice}</div>}
        {emailSettings && <section className="payment-settings-form">
          <label>SMTP server<input value={emailSettings.server} onChange={(event) => setEmailSettings({...emailSettings,server:event.target.value})}/></label>
          <label>Port<input type="number" value={emailSettings.port} onChange={(event) => setEmailSettings({...emailSettings,port:Number(event.target.value)})}/></label>
          <label>Uživatelské jméno<input value={emailSettings.username} onChange={(event) => setEmailSettings({...emailSettings,username:event.target.value})}/></label>
          <label>E-mail odesílatele<input type="email" value={emailSettings.senderEmail} onChange={(event) => setEmailSettings({...emailSettings,senderEmail:event.target.value})}/></label>
          <label>Šifrování<select value={emailSettings.encryption} onChange={(event) => setEmailSettings({...emailSettings,encryption:event.target.value})}><option>STARTTLS</option><option>TLS</option><option>Bez šifrování</option></select></label>
          <label>Název zabezpečeného záznamu<input value={emailSettings.credentialName} onChange={(event) => setEmailSettings({...emailSettings,credentialName:event.target.value})}/></label>
          <label className="wide">SMTP heslo<input type="password" value={emailSettings.password ?? ""} placeholder={emailSettings.passwordConfigured ? "Heslo je bezpečně uložené" : "Zadejte heslo"} onChange={(event) => setEmailSettings({...emailSettings,password:event.target.value})}/></label>
          <p className="wide settings-help">Heslo se ukládá pouze do Windows Credential Manageru a nikdy do databáze.</p>
          <footer className="form-actions wide"><button className="primary" disabled={saving} onClick={saveEmailSettings}><Save /> Uložit SMTP</button></footer>
        </section>}
      </div></Shell>;
    }
    if (settingsSection === "receipts") {
      return <Shell {...shellUpdater} active="Nastavení" user={user} onNavigate={navigate} onLogout={leaveToLogin}><div className="page payment-settings-page">
        <header className="page-header"><div><small>Nastavení</small><h1>Doklady o zaplacení</h1></div><button onClick={() => setSettingsSection(null)}><ArrowLeft /> Zpět na nastavení</button></header>
        {error && <div className="message error">{error}</div>}{notice && <div className="message success">{notice}</div>}
        {receiptSettings && <section className="payment-settings-form">
          <label className="checkbox wide"><input type="checkbox" checked={receiptSettings.automaticCreation} onChange={(event) => setReceiptSettings({...receiptSettings,automaticCreation:event.target.checked})}/>Automaticky vytvářet po úplné úhradě</label>
          <label className="checkbox wide"><input type="checkbox" checked={receiptSettings.automaticSending} onChange={(event) => setReceiptSettings({...receiptSettings,automaticSending:event.target.checked})}/>Automaticky odeslat doklad po úplné úhradě</label>
          <label>Pojistník<input value={receiptSettings.policyholder} onChange={(event) => setReceiptSettings({...receiptSettings,policyholder:event.target.value})}/></label>
          <label>Číslo smlouvy<input value={receiptSettings.contractNumber} onChange={(event) => setReceiptSettings({...receiptSettings,contractNumber:event.target.value})}/></label>
          <label className="wide">Předmět e-mailu<input value={receiptSettings.emailSubject} onChange={(event) => setReceiptSettings({...receiptSettings,emailSubject:event.target.value})}/></label>
          <label className="wide">Text e-mailu<textarea value={receiptSettings.emailBody} onChange={(event) => setReceiptSettings({...receiptSettings,emailBody:event.target.value})}/></label>
          <p className="wide settings-help">Loga, podpis a razítko jsou převzaty z ověřené Access předlohy a nejsou uživatelsky měněny.</p>
          <footer className="form-actions wide"><button className="primary" disabled={saving} onClick={saveReceiptSettings}><Save /> Uložit nastavení</button></footer>
        </section>}
      </div></Shell>;
    }
    if (settingsSection === "tariffs") {
      return (
        <Shell {...shellUpdater} active="Nastavení" user={user} onNavigate={navigate} onLogout={leaveToLogin}>
          <div className="page tariff-settings-page">
            <header className="page-header">
              <div><small>Nastavení</small><h1>Sazby pojistného</h1></div>
              <div className="detail-actions">
                <button onClick={() => { setSettingsSection(null); setTariffForm(null); }}><ArrowLeft /> Zpět na nastavení</button>
                {role === "Správce" && <button className="primary" onClick={() => setTariffForm(emptyTariffRate())}>Přidat novou sazbu</button>}
              </div>
            </header>
            {error && <div className="message error">{error}</div>}
            {notice && <div className="message success">{notice}</div>}
            {tariffForm && (
              <section className="tariff-form">
                <header><div><small>Sazba pojistného</small><h2>{tariffForm.id ? "Upravit sazbu" : "Nová sazba"}</h2></div></header>
                <div className="tariff-form-grid">
                  <label>Pojistná částka
                    <input type="number" min="1" value={tariffForm.insuredAmount} onChange={(event) => setTariffForm({ ...tariffForm, insuredAmount: Number(event.target.value) })} />
                  </label>
                  <label>Kategorie
                    <select value={tariffForm.category} onChange={(event) => setTariffForm({ ...tariffForm, category: event.target.value as "A" | "B" | "C" })}>
                      <option>A</option><option>B</option><option>C</option>
                    </select>
                  </label>
                  <label className="checkbox">
                    <input type="checkbox" checked={tariffForm.lossInsurance} onChange={(event) => setTariffForm({ ...tariffForm, lossInsurance: event.target.checked })} />
                    Pojištění ztráty
                  </label>
                  <label>Roční pojistné
                    <input type="number" min="0" value={tariffForm.annualPremium} onChange={(event) => setTariffForm({ ...tariffForm, annualPremium: Number(event.target.value) })} />
                  </label>
                  <label>Platnost od
                    <input type="date" value={tariffForm.validFrom} onChange={(event) => setTariffForm({ ...tariffForm, validFrom: event.target.value })} />
                  </label>
                  <label>Platnost do
                    <input type="date" value={tariffForm.validTo ?? ""} onChange={(event) => setTariffForm({ ...tariffForm, validTo: event.target.value || undefined })} />
                  </label>
                  <label className="checkbox">
                    <input type="checkbox" checked={tariffForm.active} onChange={(event) => setTariffForm({ ...tariffForm, active: event.target.checked })} />
                    Aktivní sazba
                  </label>
                  <label className="wide">Poznámka
                    <textarea value={tariffForm.note ?? ""} onChange={(event) => setTariffForm({ ...tariffForm, note: event.target.value })} />
                  </label>
                </div>
                <footer className="form-actions">
                  <button className="primary" disabled={tariffsLoading} onClick={saveTariffRate}>Uložit sazbu</button>
                  <button disabled={tariffsLoading} onClick={() => setTariffForm(null)}>Zrušit</button>
                </footer>
              </section>
            )}
            <section className="tariff-table">
              <table>
                <thead><tr>
                  <th>Pojistná částka</th><th>Kategorie</th><th>Pojištění ztráty</th>
                  <th>Roční pojistné</th><th>Platnost od</th><th>Platnost do</th>
                  <th>Stav</th><th>Poznámka</th><th>Akce</th>
                </tr></thead>
                <tbody>
                  {tariffRates.map((rate) => (
                    <tr key={rate.id}>
                      <td>{displayCurrency(rate.insuredAmount)}</td><td>{rate.category}</td>
                      <td><LossStatus value={rate.lossInsurance ? "1" : "0"} /></td>
                      <td>{displayCurrency(rate.annualPremium)}</td><td>{displayDate(rate.validFrom)}</td>
                      <td>{displayDate(rate.validTo)}</td><td><span className={rate.active ? "rate-active" : "rate-inactive"}>{rate.active ? "Aktivní" : "Neaktivní"}</span></td>
                      <td>{display(rate.note)}</td>
                      <td className="tariff-actions">
                        {role === "Správce" && <>
                          <button onClick={() => setTariffForm({ ...rate })}>Upravit</button>
                          <button onClick={() => setTariffForm({ ...rate, validTo: rate.validTo ?? "" })}>Ukončit platnost</button>
                          <button onClick={() => toggleTariffRate(rate)}>{rate.active ? "Deaktivovat" : "Aktivovat"}</button>
                        </>}
                      </td>
                    </tr>
                  ))}
                  {tariffsLoading && <tr><td colSpan={9} className="empty-row">Načítám sazby…</td></tr>}
                </tbody>
              </table>
            </section>
          </div>
        </Shell>
      );
    }
    return (
      <Shell {...shellUpdater} active="Nastavení" user={user} onNavigate={navigate} onLogout={leaveToLogin}>
        <div className="page">
          <header className="page-header"><div><small>Modul</small><h1>Nastavení</h1></div></header>
          <p className="page-intro">Možnosti nastavení budou dostupné v některé z dalších verzí.</p>
          <section className="settings-grid">
            {SETTINGS_MODULES.map((module) => (
              <article
                key={module.id}
                className={module.enabled ? "enabled" : ""}
                onClick={module.id === "tariffs" ? openTariffSettings : module.id === "payments" ? openPaymentSettings : module.id === "email" ? openEmailSettings : module.id === "receipts" ? openReceiptSettings : module.id === "updates" ? () => setSettingsSection("updates") : module.id === "backups" ? () => navigate("Správa záloh") : undefined}
              >
                {module.id === "payments" ? <CreditCard /> : <Settings />}<strong>{module.label}</strong>
                <small>{module.enabled ? "Otevřít nastavení" : "Připravujeme"}</small>
              </article>
            ))}
          </section>
        </div>
      </Shell>
    );
  }

  if (screen === "O programu") {
    return (
      <Shell {...shellUpdater} active="O programu" user={user} onNavigate={navigate} onLogout={leaveToLogin}>
        <div className="page">
          <header className="page-header"><div><small>Pojištění</small><h1>O programu</h1></div></header>
          <section className="about-card">
            <img src="/logo.png" alt="Federace vlakových čet" />
            <div>
              <h2>Pojištění</h2><p>Aplikace pro správu pojištění členů Federace vlakových čet.</p>
              <strong>Verze {dashboard?.programVersion ?? "0.17.0"}</strong>
              <dl className="build-metadata">
                <div><dt>Git tag</dt><dd>{dashboard?.gitTag ?? "neuvedeno"}</dd></div>
                <div><dt>Commit SHA</dt><dd>{dashboard?.commitSha ?? "lokální sestavení"}</dd></div>
                <div><dt>Datum buildu</dt><dd>{dashboard?.buildDate ? displayDateTime(dashboard.buildDate) : "neuvedeno"}</dd></div>
                <div><dt>Poslední kontrola aktualizací</dt><dd>{lastUpdateCheck}</dd></div>
                <div><dt>Stav updateru</dt><dd>{updaterStatus}</dd></div>
              </dl>
              <div className="about-update-controls">
                <label className="checkbox">
                  <input
                    type="checkbox"
                    checked={updateCheckEnabled}
                    onChange={(event) => changeUpdateCheckEnabled(event.target.checked)}
                  />
                  Kontrolovat aktualizace při spuštění
                </label>
                <button className="primary" disabled={updateChecking} onClick={() => void checkForUpdates(true)}>
                  {updateChecking ? "Kontroluji…" : "Vyhledat aktualizace"}
                </button>
              </div>
              {updateMessage && <div className="message info">{updateMessage}</div>}
            </div>
          </section>
        </div>
      </Shell>
    );
  }

  if (screen === "Správa záloh") {
    return (
      <Shell {...shellUpdater} active="Správa záloh" user={user} onNavigate={navigate} onLogout={leaveToLogin}>
        <div className="page backup-management-page">
          <header className="page-header">
            <div><small>Soubor</small><h1>Správa záloh</h1></div>
            <div className="detail-actions">
              <button className="primary" disabled={backupBusy} onClick={() => void createDatabaseBackup()}><HardDriveDownload /> Vytvořit zálohu</button>
              <button disabled={backupBusy} onClick={() => void restoreDatabaseBackup()}><Upload /> Obnovit ze zálohy</button>
            </div>
          </header>
          <p className="page-intro">Přehled ručně vytvořených a nouzových záloh dostupných v tomto počítači.</p>
          {error && <div className="message error">{error}</div>}
          {backupsLoading ? (
            <div className="panel">Načítám zálohy…</div>
          ) : backups.length === 0 ? (
            <div className="empty-state"><FolderArchive /><h2>Zatím nejsou evidovány žádné zálohy</h2></div>
          ) : (
            <div className="table-wrap backup-table-wrap">
              <table>
                <thead><tr><th>Název souboru</th><th>Datum vytvoření</th><th>Verze aplikace</th><th>Verze databáze</th><th>Počet členů</th><th>Velikost</th><th>Kontrolní součet</th><th></th></tr></thead>
                <tbody>
                  {backups.map((backup) => (
                    <tr key={backup.path}>
                      <td><strong>{backup.fileName}</strong>{backup.emergency && <small className="backup-emergency">Nouzová záloha</small>}</td>
                      <td>{displayBackupDate(backup.createdAt)}</td>
                      <td>{backup.applicationVersion}</td>
                      <td>{backup.schemaVersion}</td>
                      <td>{backup.memberCount.toLocaleString("cs-CZ")}</td>
                      <td>{displayFileSize(backup.databaseSize)}</td>
                      <td><code title={backup.checksum}>{backup.checksum.slice(0, 12)}…</code></td>
                      <td><button disabled={backupBusy} onClick={() => void restoreDatabaseBackup(backup.path)}>Obnovit</button></td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </Shell>
    );
  }

  if (screen === "Archiv") {
    const pages = Math.max(1, Math.ceil(archivePage.total / archivePage.pageSize));
    const detailFields: Array<[keyof Member, string, boolean?]> = [
      ["code", "Kód OC"],
      ["registrationNumber", "Evidenční číslo"],
      ["insured", "Pojištěnec"],
      ["personalId", "Rodné číslo"],
      ["affiliation", "Odborová příslušnost"],
      ["insuranceFrom", "Pojištění od", true],
      ["actualTermination", "Skutečné ukončení", true],
      ["category", "Kategorie"],
      ["loss", "Pojištění ztráty"],
      ["annualPremium", "Roční pojistné"],
      ["premium", "Pojistné"],
      ["actualPayment", "Skutečně uhrazeno"],
      ["note", "Poznámka"],
    ];
    if (selectedArchiveMember) {
      return (
        <Shell {...shellUpdater} active="Archiv" user={user} onNavigate={navigate} onLogout={leaveToLogin}>
          <div className="page member-detail-page">
            <header className="page-header">
              <div><small>Archiv / {archiveYear}</small><h1>Detail člena</h1></div>
              <button onClick={() => setSelectedArchiveMember(null)}>
                <ArrowLeft /> Zpět na rok {archiveYear}
              </button>
            </header>
            <section className="member-detail-inline">
              <h2>{display(selectedArchiveMember.insured)}</h2>
              <div className="detail-grid">
                {detailFields.map(([key, label, date]) => (
                  <div className={key === "note" ? "wide" : ""} key={key}>
                    <small>{label}</small>
                    <strong>{detailValue(selectedArchiveMember, key, date)}</strong>
                  </div>
                ))}
              </div>
              <footer><span><ShieldCheck /> Pouze pro čtení</span></footer>
            </section>
          </div>
        </Shell>
      );
    }
    if (archiveYear !== null) {
      return (
        <Shell {...shellUpdater} active="Archiv" user={user} onNavigate={navigate} onLogout={leaveToLogin}>
          <div className="page members-page">
            <section className="members-panel">
              <header className="members-header archive-header">
                <div><small>Archiv pojištění</small><h1>Rok {archiveYear}</h1></div>
                <button onClick={() => setArchiveYear(null)}><ArrowLeft /> Zpět na roky</button>
              </header>
              {error && <div className="message error">{error}</div>}
              <form className="member-search" onSubmit={searchArchive}>
                <Search />
                <input
                  value={archiveSearch}
                  onChange={(event) => setArchiveSearch(event.target.value)}
                  aria-label={`Hledat v archivu roku ${archiveYear}`}
                />
                <button type="submit" className="primary">Hledat</button>
              </form>
              <FilterBar
                filters={archiveFilters}
                onChange={setArchiveFilters}
                onApply={() => loadArchiveMembers(archiveYear, 1)}
              />
              <div className="member-count">
                <span>{activeArchiveSearch ? `Výsledky hledání: ${activeArchiveSearch}` : `Záznamy roku ${archiveYear}`}</span>
                <strong>{new Intl.NumberFormat("cs-CZ").format(archivePage.total)}</strong>
              </div>
              <div className="members-table">
                <table>
                  <thead><tr>
                    <th>Kód OC</th><th>Evidenční číslo</th><th>Pojištěnec</th><th>Rodné číslo</th>
                    <th>Odborová příslušnost</th><th>Pojištění od</th><th>Skutečné ukončení</th><th>Kategorie</th>
                    <th>Pojištění ztráty</th><th>Roční pojistné</th><th>Pojistné</th>
                    <th>Skutečně uhrazeno</th><th>Poznámka</th>
                  </tr></thead>
                  <tbody>
                    {!archiveLoading && archivePage.members.map((member) => (
                      <tr key={member.rowId} onDoubleClick={() => openArchiveMember(member.rowId)}>
                        <td>{display(member.code)}</td>
                        <td><button className="member-link" onClick={() => openArchiveMember(member.rowId)}>{display(member.registrationNumber)}</button></td>
                        <td><span className="member-with-warning">{display(member.insured)} <PaymentWarning member={member} /></span></td><td>{display(member.personalId)}</td>
                        <td>{display(member.affiliation)}</td><td>{displayDate(member.insuranceFrom)}</td>
                        <td>{displayDate(member.actualTermination)}</td><td>{display(member.category)}</td>
                        <td><LossStatus value={member.loss} /></td><td>{displayCurrency(member.annualPremium)}</td>
                        <td>{displayCurrency(member.premium)}</td><td>{displayCurrency(member.actualPayment)}</td>
                        <td className="note-cell">{display(member.note)}</td>
                      </tr>
                    ))}
                    {archiveLoading && <tr><td colSpan={13} className="empty-row">Načítám archiv…</td></tr>}
                    {!archiveLoading && archivePage.members.length === 0 && (
                      <tr><td colSpan={13} className="empty-row">Žádný odpovídající záznam.</td></tr>
                    )}
                  </tbody>
                </table>
              </div>
              <footer className="pagination">
                <span>Strana {archivePage.page} z {pages}</span>
                <div>
                  <button disabled={archivePage.page <= 1 || archiveLoading} onClick={() => loadArchiveMembers(archiveYear, archivePage.page - 1)}>
                    <ChevronLeft /> Předchozí
                  </button>
                  <button disabled={archivePage.page >= pages || archiveLoading} onClick={() => loadArchiveMembers(archiveYear, archivePage.page + 1)}>
                    Další <ChevronRight />
                  </button>
                </div>
              </footer>
            </section>
          </div>
        </Shell>
      );
    }
    return (
      <Shell {...shellUpdater} active="Archiv" user={user} onNavigate={navigate} onLogout={leaveToLogin}>
        <div className="page archive-page">
          <header className="page-header"><div><small>Pojištění</small><h1>Archiv pojištění podle roků</h1></div></header>
          {error && <div className="message error">{error}</div>}
          <section className="archive-years">
            {archiveYears.map((item) => (
              <button key={item.year} onClick={() => selectArchiveYear(item.year)}>
                <Archive />
                <strong>{String(item.year).padStart(4, "0")}</strong>
                <span>{new Intl.NumberFormat("cs-CZ").format(item.recordCount)} záznamů</span>
                {item.uniqueMemberCount !== undefined && item.uniqueMemberCount !== null && (
                  <small>{new Intl.NumberFormat("cs-CZ").format(item.uniqueMemberCount)} členů</small>
                )}
              </button>
            ))}
            {archiveLoading && <p>Načítám archiv…</p>}
          </section>
        </div>
      </Shell>
    );
  }

  if (screen === "Doklady o zaplacení") {
    return (
      <Shell {...shellUpdater} active="Doklady o zaplacení" user={user} onNavigate={navigate} onLogout={leaveToLogin}>
        <div className="page">
          <header className="page-header"><div><small>Dokumenty</small><h1>Doklady o zaplacení</h1></div></header>
          {error && <div className="message error">{error}</div>}
          {notice && <div className="message success">{notice}</div>}
          <section className="agenda-workspace">
            <h2>Vytvořit doklad</h2>
            <form className="search-bar" onSubmit={searchAgendaMembers}>
              <Search /><input value={agendaSearch} onChange={(event) => setAgendaSearch(event.target.value)} placeholder="Jméno, evidenční číslo, kód OC nebo rodné číslo" />
              <button className="primary">Vyhledat člena</button>
            </form>
            {agendaMembers.length > 0 && <div className="agenda-member-results"><table><thead><tr><th>Evidenční číslo</th><th>Člen</th><th>Kód OC</th><th>Rok</th><th>Uhrazeno</th><th>Stav</th><th></th></tr></thead><tbody>
              {agendaMembers.map((member) => <tr key={member.rowId}><td>{display(member.registrationNumber)}</td><td>{member.insured}</td><td>{display(member.code)}</td><td>{insuranceYear(member)}</td><td>{displayCurrency(member.actualPayment)}</td><td>{paymentSummary(member).label}</td><td><button onClick={() => void selectAgendaMember(member, "receipt")}>Vybrat</button></td></tr>)}
            </tbody></table></div>}
            {selectedMember && <div className="agenda-selected-member">
              <MemberHeading member={selectedMember} />
              {paymentDocumentBasis && <section className="payment-summary-cards">
                <div><span>Pojistný rok</span><strong>{paymentDocumentBasis.insuranceYear}</strong></div>
                <div><span>Předepsané pojistné</span><strong>{displayCurrency(paymentDocumentBasis.prescribedPremium)}</strong></div>
                <div><span>Skutečně uhrazeno</span><strong>{displayCurrency(paymentDocumentBasis.paidAmount)}</strong></div>
                <div><span>Stav</span><strong>{paymentSummary(selectedMember).label}</strong></div>
              </section>}
              <button className="primary" disabled={saving || !paymentDocumentBasis} onClick={() => void createMemberReceipt(selectedMember)}><Plus /> Vytvořit doklad</button>
              <div className="claims-table"><table><thead><tr><th>Datum vystavení</th><th>Datum úhrady</th><th>Rok</th><th>Částka</th><th>Stav</th><th>Akce</th></tr></thead><tbody>
                {memberReceipts.map((receipt) => <tr key={receipt.id}><td>{displayDate(receipt.issuedOn)}</td><td>{displayDate(receipt.paidOn)}</td><td>{receipt.insuranceYear}</td><td>{displayCurrency(receipt.amount)}</td><td>{receipt.status}</td><td className="row-actions"><button title="Náhled" onClick={() => void receiptAction("open_receipt_pdf", receipt)}><FileText /></button><button title="Tisk" onClick={() => void receiptAction("open_receipt_pdf", receipt, true)}><Printer /></button><button title="Export PDF" onClick={() => void receiptAction("export_receipt_pdf", receipt)}><Upload /></button></td></tr>)}
                {memberReceipts.length === 0 && <tr><td colSpan={6} className="empty-row">Člen zatím nemá vystavený doklad.</td></tr>}
              </tbody></table></div>
            </div>}
          </section>
          <h2>Vystavené doklady</h2>
          <form className="search-bar" onSubmit={(event) => { event.preventDefault(); void loadReceipts(undefined, receiptSearch); }}>
            <Search /><input value={receiptSearch} onChange={(event) => setReceiptSearch(event.target.value)} placeholder="Jméno, evidenční číslo, rok nebo e-mail" />
            <button className="primary">Vyhledat</button>
          </form>
          <div className="claims-table receipts-table"><table>
            <thead><tr><th>Evidenční číslo</th><th>Jméno a příjmení</th><th>Rok</th><th>Datum úhrady</th><th>Datum vystavení</th><th>Částka</th><th>Doklad</th><th>E-mail</th><th>Akce</th></tr></thead>
            <tbody>{receipts.map((receipt) => <tr key={receipt.id}>
              <td>{receipt.registrationNumber}</td><td>{receipt.memberName}</td><td>{receipt.insuranceYear}</td><td>{displayDate(receipt.paidOn)}</td><td>{displayDate(receipt.issuedOn)}</td><td>{displayCurrency(receipt.amount)}</td><td>{receipt.status}</td><td>{receipt.emailStatus}</td>
              <td className="row-actions">
                <button title="Otevřít detail člena" onClick={() => { setScreen("Seznam"); void openMember(receipt.memberRowId); }}><Users /></button>
                <button title="Náhled" onClick={() => void receiptAction("open_receipt_pdf", receipt)}><FileText /></button>
                <button title="Tisk" onClick={() => void receiptAction("open_receipt_pdf", receipt, true)}><Printer /></button>
                <button title="Export PDF" onClick={() => void receiptAction("export_receipt_pdf", receipt)}><Upload /></button>
                <button title={receipt.emailStatus === "Odeslán" ? "Odeslat znovu" : "Odeslat e-mailem"} onClick={() => void receiptAction("send_receipt_email", receipt)}><Mail /></button>
              </td>
            </tr>)}{receipts.length === 0 && <tr><td colSpan={9} className="empty-row">Nebyly nalezeny žádné vystavené doklady.</td></tr>}</tbody>
          </table></div>
        </div>
      </Shell>
    );
  }

  if (screen === "Seznam") {
    const pages = Math.max(1, Math.ceil(memberPage.total / memberPage.pageSize));
    if (selectedMember) {
      const tabs = [
        ["overview", "Přehled"],
        ["personal", "Osobní údaje"],
        ["organization", "Organizace"],
        ["contact", "Kontakt"],
        ["insurance", "Pojištění"],
        ["payments", "💰 Platby"],
        ["receipts", "📄 Doklady o zaplacení"],
        ["claims", "Pojistné události"],
        ["history", "Historie"],
        ["notes", "Poznámky"],
      ] as const;
      return (
        <Shell {...shellUpdater} active="Seznam" user={user} onNavigate={navigate} onLogout={leaveToLogin}>
          <div className="page member-profile-page">
            <header className="page-header">
              <div><small>Seznam pojištěnců</small><h1>Detail člena</h1></div>
            </header>
            {error && <div className="message error">{error}</div>}
            {notice && <div className="message success">{notice}</div>}
            <section className="member-profile">
              <MemberHeading member={selectedMember} />
              <section className="member-action-center" aria-label="Akce člena">
                <h2>Akce člena</h2>
                <div className="member-actions">
                  {role === "Správce" && (
                    <>
                      <button className="action-edit" title="Upravit členské údaje" aria-label="Upravit člena" onClick={() => startMemberEdit(selectedMember)}><Pencil /> Upravit člena</button>
                      <button className="action-claim" title="Založit nový případ pojistné události" aria-label="Nová pojistná událost" onClick={() => openClaimForMember(selectedMember)}><Plus /> Nová pojistná událost</button>
                      <button className="action-payment" title="Připravit příkaz k úhradě" aria-label="Vygenerovat příkaz k úhradě" onClick={() => openPaymentForMember(selectedMember)}><CreditCard /> Příkaz k úhradě</button>
                    </>
                  )}
                  <button className="action-neutral action-back" title="Vrátit se na seznam pojištěnců" aria-label="Zpět na seznam" onClick={closeMemberDetail}><ArrowLeft /> Zpět na seznam</button>
                </div>
              </section>
              {editingMember && memberEdit && (
                <section className="member-edit-panel">
                  <header><div><small>Aktuální záznam</small><h2>Upravit člena</h2></div></header>
                  <div className="member-edit-grid">
                    <label>Titul před jménem<input value={memberEdit.title} onChange={(event) => setMemberEdit({ ...memberEdit, title: event.target.value })} /></label>
                    <label>Jméno<input value={memberEdit.firstName} onChange={(event) => setMemberEdit({ ...memberEdit, firstName: event.target.value })} /></label>
                    <label>Příjmení<input value={memberEdit.lastName} onChange={(event) => setMemberEdit({ ...memberEdit, lastName: event.target.value })} /></label>
                    <label>Rodné číslo<input value={memberEdit.personalId} onChange={(event) => setMemberEdit({ ...memberEdit, personalId: formatPersonalId(event.target.value) })} /></label>
                    <label>Evidenční číslo<input type="number" value={memberEdit.registrationNumber ?? ""} onChange={(event) => setMemberEdit({ ...memberEdit, registrationNumber: event.target.value ? Number(event.target.value) : null })} /></label>
                    <label>Základní organizace<input value={memberEdit.organization} onChange={(event) => setMemberEdit({ ...memberEdit, organization: event.target.value })} /></label>
                    <label>Odborová příslušnost
                      <select value={memberEdit.affiliation} onChange={(event) => {
                        const affiliation = event.target.value;
                        setMemberEdit({ ...memberEdit, affiliation, code: affiliation === "FVČ" ? "1" : "2" });
                      }}>
                        <option>FVČ</option><option>FV</option>
                      </select>
                    </label>
                    <label>Kód OC<input value={memberEdit.code} readOnly /></label>
                    <label>Adresa<input value={memberEdit.address} onChange={(event) => setMemberEdit({ ...memberEdit, address: event.target.value })} /></label>
                    <label>Obec<input value={memberEdit.city} onChange={(event) => setMemberEdit({ ...memberEdit, city: event.target.value })} /></label>
                    <label>PSČ<input value={memberEdit.postalCode} onChange={(event) => setMemberEdit({ ...memberEdit, postalCode: formatPostalCode(event.target.value) })} /></label>
                    <label>Stát<input value={memberEdit.country} onChange={(event) => setMemberEdit({ ...memberEdit, country: event.target.value })} /></label>
                    <label>E-mail<input type="email" value={memberEdit.email} onChange={(event) => setMemberEdit({ ...memberEdit, email: event.target.value })} /></label>
                    <label>Skutečně uhrazeno (Kč)<input type="number" value={memberEdit.actualPayment ?? ""} onChange={(event) => setMemberEdit({ ...memberEdit, actualPayment: event.target.value ? Number(event.target.value) : null })} /></label>
                    <label>Skutečné ukončení<input type="date" value={memberEdit.actualTermination} onChange={(event) => setMemberEdit({ ...memberEdit, actualTermination: event.target.value })} /></label>
                    <label className="wide">Poznámka<textarea value={memberEdit.note} onChange={(event) => setMemberEdit({ ...memberEdit, note: event.target.value })} /></label>
                  </div>
                  <footer className="form-actions">
                    <button className="primary" disabled={saving} onClick={saveMemberEdit}>Uložit změny</button>
                    <button disabled={saving} onClick={() => { setEditingMember(false); setMemberEdit(null); }}>Zrušit</button>
                  </footer>
                </section>
              )}
              {!editingMember && <>
              <nav className="detail-tabs" aria-label="Části detailu člena" role="tablist">
                {tabs.map(([id, label]) => (
                  <button
                    key={id}
                    role="tab"
                    aria-selected={detailTab === id}
                    className={detailTab === id ? "active" : ""}
                    onClick={() => {
                      setDetailTab(id);
                      setHistoryMember(null);
                      if (id === "claims") void loadMemberClaims(selectedMember);
                      if (id === "receipts") void loadMemberReceiptData(selectedMember);
                    }}
                  >
                    {label}
                  </button>
                ))}
              </nav>
              {detailTab === "overview" && (
                <div className="single-detail-section"><OverviewSection member={selectedMember} /></div>
              )}
              {detailTab === "personal" && <div className="single-detail-section"><PersonalSection member={selectedMember} /></div>}
              {detailTab === "organization" && <div className="single-detail-section"><OrganizationSection member={selectedMember} /></div>}
              {detailTab === "contact" && <div className="single-detail-section"><ContactSection member={selectedMember} /></div>}
              {detailTab === "insurance" && <div className="single-detail-section"><InsuranceSection member={selectedMember} /></div>}
              {detailTab === "payments" && (
                <div className="member-payments-section">
                  <section className="payment-summary-cards">
                    <div><span>Roční pojistné</span><strong>{displayCurrency(selectedMember.premium)}</strong></div>
                    <div><span>Skutečně uhrazeno</span><strong>{displayCurrency(selectedMember.actualPayment)}</strong></div>
                    <div><span>Zbývá uhradit</span><strong>{displayCurrency(Math.max(Number(selectedMember.premium ?? 0) - Number(selectedMember.actualPayment ?? 0), 0))}</strong></div>
                    <div><span>Stav</span><strong className={`payment-${paymentSummary(selectedMember).tone}`}>{paymentSummary(selectedMember).label}</strong></div>
                  </section>
                  <header className="member-payments-header">
                    <h2>Historie plateb</h2>
                    <button className="primary" onClick={() => newMemberPayment(selectedMember)}><Plus /> Přidat platbu</button>
                  </header>
                  {memberPaymentForm && (
                    <section className="member-payment-form">
                      <h3>{memberPaymentForm.id ? "Upravit platbu" : "Nová platba"}</h3>
                      <label>Datum přijetí<input type="date" value={memberPaymentForm.receivedOn} onChange={(event) => setMemberPaymentForm({ ...memberPaymentForm, receivedOn: event.target.value })} /></label>
                      <label>Částka (Kč)<input type="number" min="1" value={memberPaymentForm.amount} onChange={(event) => setMemberPaymentForm({ ...memberPaymentForm, amount: event.target.value })} /></label>
                      <label>Způsob úhrady<select disabled={Boolean(memberPaymentForm.id)} value={memberPaymentForm.method} onChange={(event) => setMemberPaymentForm({ ...memberPaymentForm, method: event.target.value as MemberPaymentForm["method"] })}><option>Bankovní převod</option><option>Hotově</option><option>Jiné</option></select></label>
                      <label className="wide">Poznámka<textarea value={memberPaymentForm.note} onChange={(event) => setMemberPaymentForm({ ...memberPaymentForm, note: event.target.value })} /></label>
                      <footer><button className="primary" disabled={saving} onClick={saveMemberPayment}><Save /> Uložit platbu</button><button disabled={saving} onClick={() => setMemberPaymentForm(null)}>Zrušit</button></footer>
                    </section>
                  )}
                  <div className="claims-table">
                    <table>
                      <thead><tr><th>Datum přijetí</th><th>Částka</th><th>Pojistný rok</th><th>Způsob úhrady</th><th>Variabilní symbol</th><th>Poznámka</th><th>Stav</th><th>Akce</th></tr></thead>
                      <tbody>
                        {memberPayments.map((payment) => <tr key={payment.id}>
                          <td>{displayDate(payment.receivedOn)}</td><td>{displayCurrency(payment.amount)}</td><td>{payment.insuranceYear}</td><td>{payment.method}</td><td>{payment.variableSymbol}</td><td>{display(payment.note)}</td><td>{payment.status}</td>
                          <td className="row-actions"><button title="Upravit platbu" onClick={() => editMemberPayment(selectedMember, payment)}><Pencil /></button><button title="Odstranit platbu" onClick={() => removeMemberPayment(payment)}><CircleX /></button></td>
                        </tr>)}
                        {memberPayments.length === 0 && <tr><td colSpan={8} className="empty-row">Člen zatím nemá evidovanou platbu.</td></tr>}
                      </tbody>
                    </table>
                  </div>
                </div>
              )}
              {detailTab === "claims" && (
                <div className="claims-section">
                  {role === "Správce" && <button className="action-claim" onClick={() => openClaimForMember(selectedMember)}><Plus /> Nová pojistná událost</button>}
                  <div className="claims-table">
                    <table>
                      <thead><tr><th>ID</th><th>Rok</th><th>Datum vzniku</th><th>Popis</th><th>Zjištěná škoda</th><th>Plnění</th><th>Stav</th></tr></thead>
                      <tbody>
                        {memberClaims.map((claim) => (
                          <tr key={claim.id}>
                            <td>{claim.id}</td><td>{claim.insuranceYear}</td><td>{displayDate(claim.occurredOn)}</td>
                            <td>{display(claim.description)}</td><td>{displayCurrency(claim.assessedDamage)}</td>
                            <td>{displayCurrency(claim.insuranceBenefit)}</td>
                            <td><span className={`claim-status ${claim.status === "Otevřená" ? "open" : "closed"}`}>{claim.status}</span></td>
                          </tr>
                        ))}
                        {!claimsLoading && memberClaims.length === 0 && <tr><td colSpan={7} className="empty-row">Člen nemá evidovanou pojistnou událost.</td></tr>}
                        {claimsLoading && <tr><td colSpan={7} className="empty-row">Načítám pojistné události…</td></tr>}
                      </tbody>
                    </table>
                  </div>
                </div>
              )}
              {detailTab === "receipts" && (
                <div className="member-payments-section">
                  <header className="member-payments-header"><h2>Doklady o zaplacení</h2><button className="primary" disabled={saving} onClick={() => void createMemberReceipt(selectedMember)}><Plus /> Vytvořit nový doklad</button></header>
                  {paymentDocumentBasis && <section className="payment-summary-cards">
                    <div><span>Pojistný rok</span><strong>{paymentDocumentBasis.insuranceYear}</strong></div>
                    <div><span>Předepsané pojistné</span><strong>{displayCurrency(paymentDocumentBasis.prescribedPremium)}</strong></div>
                    <div><span>Skutečně uhrazeno</span><strong>{displayCurrency(paymentDocumentBasis.paidAmount)}</strong></div>
                    <div><span>Číslo smlouvy</span><strong>{paymentDocumentBasis.contractNumber}</strong></div>
                  </section>}
                  <div className="claims-table"><table>
                    <thead><tr><th>Datum vystavení</th><th>Datum úhrady</th><th>Rok</th><th>Částka</th><th>Číslo smlouvy</th><th>Stav</th><th>Odeslání</th><th>E-mail</th><th>Akce</th></tr></thead>
                    <tbody>{memberReceipts.map((receipt) => <tr key={receipt.id}>
                      <td>{displayDate(receipt.issuedOn)}</td><td>{displayDate(receipt.paidOn)}</td><td>{receipt.insuranceYear}</td><td>{displayCurrency(receipt.amount)}</td><td>{receipt.contractNumber}</td><td>{receipt.status}</td><td>{receipt.emailStatus}{receipt.sentAt ? ` · ${displayDateTime(receipt.sentAt)}` : ""}</td><td>{display(receipt.recipientEmail)}</td>
                      <td className="row-actions"><button title="Náhled" onClick={() => void receiptAction("open_receipt_pdf", receipt)}><FileText /></button><button title="Tisk" onClick={() => void receiptAction("open_receipt_pdf", receipt, true)}><Printer /></button><button title="Export PDF" onClick={() => void receiptAction("export_receipt_pdf", receipt)}><Upload /></button><button title={receipt.emailStatus === "Odeslán" ? "Odeslat znovu" : "Odeslat e-mailem"} onClick={() => void receiptAction("send_receipt_email", receipt)}><Mail /></button></td>
                    </tr>)}{memberReceipts.length === 0 && <tr><td colSpan={9} className="empty-row">Člen zatím nemá vystavený doklad.</td></tr>}</tbody>
                  </table></div>
                </div>
              )}
              {detailTab === "notes" && <div className="single-detail-section"><NotesSection member={selectedMember} /></div>}
              {detailTab === "history" && !historyMember && (
                <div className="member-history-sections">
                  <section className="history-table">
                    <h2>Auditní historie</h2>
                    <table>
                      <thead><tr><th>Datum a čas</th><th>Uživatel</th><th>Akce</th><th>Výsledek</th></tr></thead>
                      <tbody>
                        {memberAuditHistory.map((entry, index) => (
                          <tr key={`${entry.occurredAt}-${index}`}>
                            <td>{displayDateTime(entry.occurredAt)}</td><td>{display(entry.user)}</td>
                            <td>{entry.operation === "INSERT" ? "Vytvořen člen" : entry.operation === "UPDATE" ? "Upraven člen" : entry.operation === "INSERT_CLAIM" ? "Přidána pojistná událost" : entry.operation.startsWith("INSERT:") ? `Přidána platba ${displayCurrency(entry.operation.split(":")[1])}` : entry.operation.startsWith("UPDATE:") ? `Upravena platba ${displayCurrency(entry.operation.split(":")[1])}` : entry.operation.startsWith("DELETE:") ? `Odstraněna platba ${displayCurrency(entry.operation.split(":")[1])}` : entry.operation}</td>
                            <td><span className={entry.result === "OK" ? "audit-ok" : "audit-error"}>{entry.result}</span></td>
                          </tr>
                        ))}
                        {memberAuditHistory.length === 0 && <tr><td colSpan={4} className="empty-row">Pro tohoto člena nejsou dostupné auditní záznamy.</td></tr>}
                      </tbody>
                    </table>
                  </section>
                  <section className="history-table">
                  <h2>Historie pojištění</h2>
                  <table>
                    <thead><tr>
                      <th>Rok</th><th>Pojistná částka</th><th>Roční pojistné</th>
                      <th>Uhrazeno</th><th>Stav</th><th>Kategorie</th><th>Pojištění ztráty</th>
                    </tr></thead>
                    <tbody>
                      {memberHistory.map((item) => (
                        <tr key={item.rowId} onClick={() => setHistoryMember(item)}>
                          <td>{insuranceYear(item)}</td>
                          <td>{displayCurrency(item.annualPremium)}</td>
                          <td>{displayCurrency(item.premium)}</td>
                          <td>{displayCurrency(item.actualPayment)}</td>
                          <td>{paymentSummary(item).label}</td>
                          <td>{display(item.category)}</td>
                          <td><LossStatus value={item.loss} /></td>
                        </tr>
                      ))}
                      {memberHistory.length === 0 && (
                        <tr><td colSpan={7} className="empty-row">Pro tohoto člena nejsou dostupné starší pojistné záznamy.</td></tr>
                      )}
                    </tbody>
                  </table>
                  </section>
                </div>
              )}
              {detailTab === "history" && historyMember && (
                <div className="historical-detail">
                  <header>
                    <div><small>Historický záznam – pouze pro čtení</small><h2>Pojištění za rok {insuranceYear(historyMember)}</h2></div>
                    <button onClick={() => setHistoryMember(null)}><ArrowLeft /> Zpět na historii</button>
                  </header>
                  <div className="insurance-detail-grid">
                    <InsuranceSection member={historyMember} />
                    <OrganizationSection member={historyMember} />
                    <NotesSection member={historyMember} />
                  </div>
                  <p className="readonly-note"><ShieldCheck /> Historický záznam je pouze pro čtení.</p>
                </div>
              )}
              </>}
            </section>
          </div>
        </Shell>
      );
    }
    return (
      <Shell {...shellUpdater} active="Seznam" user={user} onNavigate={navigate} onLogout={leaveToLogin}>
        <div className="page members-page">
        <section className="members-panel">
          <header className="members-header">
            <div>
              <small>Přehled pojištěnců</small>
              <h1>Seznam všech členů</h1>
              <p>Pojištění {dashboard?.activeInsuranceYear ?? "—"}</p>
            </div>
          </header>
          {error && <div className="message error">{error}</div>}
          <form className="member-search" onSubmit={searchMembers}>
            <Search />
            <input
              value={memberSearch}
              onChange={(event) => setMemberSearch(event.target.value)}
              aria-label="Hledat v seznamu členů"
            />
            <button type="submit" className="primary">Hledat</button>
          </form>
          <FilterBar
            filters={memberFilters}
            onChange={setMemberFilters}
            onApply={() => loadMembers(1)}
          />
          <div className="member-count">
            <span>{activeSearch ? `Výsledky hledání: ${activeSearch}` : "Všechny záznamy"}</span>
            <strong>{new Intl.NumberFormat("cs-CZ").format(memberPage.total)}</strong>
          </div>
          <div className="members-table" ref={membersTableRef}>
            <table>
              <thead>
                <tr>
                  <th>Kód OC</th>
                  <th>Evidenční číslo</th>
                  <th>Pojištěnec</th>
                  <th>Rodné číslo</th>
                  <th>Odborová příslušnost</th>
                  <th>Pojištění od</th>
                  <th>Skutečné ukončení</th>
                  <th>Kategorie</th>
                  <th>Pojištění ztráty</th>
                  <th>Roční pojistné</th>
                  <th>Pojistné</th>
                  <th>Skutečně uhrazeno</th>
                  <th>Poznámka</th>
                </tr>
              </thead>
              <tbody>
                {!membersLoading &&
                  memberPage.members.map((member) => (
                    <tr key={member.rowId} onDoubleClick={() => openMember(member.rowId)}>
                      <td>{display(member.code)}</td>
                      <td>
                        <button className="member-link" onClick={(event) => { event.stopPropagation(); openMember(member.rowId); }}>
                          {display(member.registrationNumber)}
                        </button>
                      </td>
                      <td>
                        <button className="member-link member-name" onClick={(event) => { event.stopPropagation(); openMember(member.rowId); }}>
                          {display(member.insured)} <PaymentWarning member={member} />
                        </button>
                      </td>
                      <td>{display(member.personalId)}</td>
                      <td>{display(member.affiliation)}</td>
                      <td>{displayDate(member.insuranceFrom)}</td>
                      <td>{displayDate(member.actualTermination)}</td>
                      <td>{display(member.category)}</td>
                      <td><LossStatus value={member.loss} /></td>
                      <td>{displayCurrency(member.annualPremium)}</td>
                      <td>{displayCurrency(member.premium)}</td>
                      <td>{displayCurrency(member.actualPayment)}</td>
                      <td className="note-cell">{display(member.note)}</td>
                    </tr>
                  ))}
                {membersLoading && (
                  <tr><td colSpan={13} className="empty-row">Načítám seznam členů…</td></tr>
                )}
                {!membersLoading && memberPage.members.length === 0 && (
                  <tr><td colSpan={13} className="empty-row">Žádný odpovídající záznam.</td></tr>
                )}
              </tbody>
            </table>
          </div>
          <footer className="pagination">
            <span>Strana {memberPage.page} z {pages}</span>
            <div>
              <button
                disabled={memberPage.page <= 1 || membersLoading}
                onClick={() => loadMembers(memberPage.page - 1)}
              >
                <ChevronLeft /> Předchozí
              </button>
              <button
                disabled={memberPage.page >= pages || membersLoading}
                onClick={() => loadMembers(memberPage.page + 1)}
              >
                Další <ChevronRight />
              </button>
            </div>
          </footer>
        </section>
        </div>
      </Shell>
    );
  }

  return (
    <Shell {...shellUpdater} active="Pojištěnci" user={user} onNavigate={navigate} onLogout={leaveToLogin}>
      <div className="page insured-screen">
      <section className="insured-panel">
        <header>
          <div>
            <small>Pojištěnci</small>
            <h1>Zadání základních údajů pojištěnců</h1>
          </div>
          <div className="registration-tools">
            <label>
              Rok
              <input
                type="number"
                value={form.registrationYear}
                onChange={(event) => update("registrationYear", Number(event.target.value))}
              />
            </label>
            <button type="button" onClick={showLastRegistration}>
              Zobrazit poslední
            </button>
          </div>
        </header>

        {error && (
          <div className="message error">
            <CircleX />
            {error}
            <button onClick={() => setError("")} aria-label="Zavřít chybové hlášení">
              <X />
            </button>
          </div>
        )}
        {notice && <div className="message success">{notice}</div>}

        <form className="insured-form" onSubmit={(event) => event.preventDefault()}>
          <label>
            Titul
            <select
              ref={titleRef}
              value={form.title}
              onChange={(event) => update("title", event.target.value)}
            >
              <option value="" />
              <option>Bc.</option>
              <option>Ing.</option>
              <option>JUDr.</option>
              <option>Mgr.</option>
            </select>
          </label>
          <label>
            Příjmení
            <input value={form.lastName} onChange={(event) => update("lastName", event.target.value)} />
          </label>
          <label>
            Jméno
            <input value={form.firstName} onChange={(event) => update("firstName", event.target.value)} />
          </label>
          <label>
            Rodné číslo
            <input
              inputMode="numeric"
              placeholder="000000/0000"
              value={form.personalId}
              onChange={(event) => update("personalId", formatPersonalId(event.target.value))}
            />
          </label>
          <label>
            Základní organizace
            <select
              value={form.organization}
              onChange={(event) => update("organization", event.target.value)}
            >
              <option value="" />
              {options.organizations.map((organization) => (
                <option key={organization}>{organization}</option>
              ))}
            </select>
          </label>
          <label>
            Odborová příslušnost
            <select
              value={form.affiliation}
              onChange={(event) => {
                const affiliation = event.target.value as "FVČ" | "FV";
                setForm((current) => ({
                  ...current,
                  affiliation,
                  code: affiliation === "FVČ" ? 1 : 2,
                  organization: "",
                }));
              }}
            >
              <option>FVČ</option>
              <option>FV</option>
            </select>
          </label>
          <label>
            Město
            <input value={form.city} onChange={(event) => update("city", event.target.value)} />
          </label>
          <label>
            Adresa
            <input value={form.address} onChange={(event) => update("address", event.target.value)} />
          </label>
          <label>
            PSČ
            <input
              inputMode="numeric"
              placeholder="000 00"
              value={form.postalCode}
              onChange={(event) => update("postalCode", formatPostalCode(event.target.value))}
            />
          </label>
          <label>
            Stát
            <select value={form.country} onChange={(event) => update("country", event.target.value)}>
              <option value="" />
              <option>Česká republika</option>
              <option>Slovenská republika</option>
            </select>
          </label>
          <label className="wide">
            Poznámka
            <textarea value={form.note} onChange={(event) => update("note", event.target.value)} />
          </label>
          <label>
            Pojištění od
            <input
              type="date"
              value={form.insuranceFrom}
              onChange={(event) => update("insuranceFrom", event.target.value)}
            />
          </label>
          <label>
            Pojištění do
            <input
              type="date"
              value={form.insuranceTo}
              onChange={(event) => update("insuranceTo", event.target.value)}
            />
          </label>
          <label>
            Pojistná částka
            <select
              value={form.annualAmount}
              onChange={(event) => update("annualAmount", Number(event.target.value))}
            >
              {(options.annualAmounts.length ? options.annualAmounts : [200_000]).map((amount) => (
                <option value={amount} key={amount}>
                  {displayCurrency(amount)}
                </option>
              ))}
            </select>
          </label>
          <label>
            Kategorie
            <select
              value={form.category}
              onChange={(event) => update("category", event.target.value as "A" | "B" | "C")}
            >
              <option>A</option>
              <option>B</option>
              <option>C</option>
            </select>
          </label>
          <label className="checkbox">
            <input
              type="checkbox"
              checked={form.loss}
              onChange={(event) => update("loss", event.target.checked)}
            />
            Pojištění ztráty
          </label>
          <label>
            Vypočtené pojistné
            <input value={displayCurrency(tariff.insuredAmount)} readOnly />
          </label>
          <label>
            Skutečně uhrazeno (Kč)
            <input
              type="number"
              value={form.actualPayment}
              onChange={(event) => update("actualPayment", event.target.value)}
            />
          </label>
          <label>
            Kód OC
            <input value={form.code} readOnly />
          </label>
          <label>
            Evidenční číslo
            <input value={registrationNumber} readOnly />
          </label>
          <label>
            E-mail
            <input
              type="email"
              value={form.email}
              onChange={(event) => update("email", event.target.value)}
            />
          </label>

          <div className="calculation wide">
            <span>Počet měsíců: <strong>{tariff.months}</strong></span>
            <span>Roční pojistné: <strong>{displayCurrency(tariff.premium)}</strong></span>
          </div>

          <div className="form-actions wide">
            <button type="button" className="primary" disabled={saving} onClick={() => save(false)}>
              Uložit a přidat další
            </button>
            <button type="button" disabled={saving} onClick={() => save(true)}>
              Uložit a zavřít
            </button>
            <button type="button" disabled={saving} onClick={cancelInsured}>
              Zrušit
            </button>
          </div>
        </form>
      </section>
      </div>
    </Shell>
  );
}
