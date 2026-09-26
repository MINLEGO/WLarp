/* Reflète les DTO Rust (camelCase). */

export type MediaType = "pdf" | "image" | "audio" | "video" | "mdx" | "unknown";

export interface FolderDto {
  id: string;
  parentId: string | null;
  name: string;
  createdAt: string;
  docCount: number;
}

export interface DocDto {
  id: string;
  folderId: string;
  title: string;
  examDate: string | null;
  excluded: boolean;
  useTranscription: boolean;
  status: string;
  createdAt: string;
  updatedAt: string;
  supportCount: number;
  questionCount: number;
  dueCount: number;
}

export interface SupportDto {
  id: string;
  docId: string;
  fileName: string;
  mediaType: MediaType;
  mime: string;
  size: number;
  relPath: string;
  createdAt: string;
  hasTranscription: boolean;
}

export interface DocDetail {
  doc: DocDto;
  supports: SupportDto[];
}

export interface ImportReport {
  created: SupportDto[];
  createdDocs: DocDto[];
  duplicates: number;
  skipped: string[];
}

export interface Settings {
  visionModel: string;
  textModel: string;
  dailyTarget: number;
  newPreviewPct: number;
  maxNewPerDay: number;
  sessionMax: number;
  reinforceEnabled: boolean;
  reinforceThreshold: number;
  reinforceMax: number;
  sessionDeadline: string;
  notifyTime: string;
  snoozeTimes: string;
  useCustomSms: boolean;
  smsProvider: string;
  smsApiKey: string;
  smsApiBase: string;
  smsSenderId: string;
  smsFromNumber: string;
  smsToNumber: string;
}

export interface ModelInfo {
  id: string;
  name: string;
  vision: boolean;
  contextLength: number;
}

export const DEFAULT_SETTINGS: Settings = {
  visionModel: "google/gemini-2.5-flash",
  textModel: "anthropic/claude-sonnet-4.5",
  dailyTarget: 30,
  newPreviewPct: 40,
  maxNewPerDay: 20,
  sessionMax: 40,
  reinforceEnabled: true,
  reinforceThreshold: 80,
  reinforceMax: 3,
  sessionDeadline: "22:30",
  notifyTime: "19:00",
  snoozeTimes: "20:30,21:30",
  useCustomSms: false,
  smsProvider: "smsfactor",
  smsApiKey: "",
  smsApiBase: "https://rest.smsfactor.com/sms/1/SMS/Message/SMS",
  smsSenderId: "",
  smsFromNumber: "",
  smsToNumber: "",
};