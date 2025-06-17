import { z } from "zod";
const privileges = ["Check", "Export", "Import", "Delete", "UsersList", "FilesUpload"] as const;
type Privilegy = typeof privileges[number];
// Схема для Privilegy (аналог enum Privilegy)
const PrivilegySchema = z.enum(privileges);
export {type Privilegy, PrivilegySchema, privileges}