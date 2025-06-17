import { DateTime } from "@/services/date";
import { z } from "zod";
import { date_time_schema } from "./date_schema";

// interface Document 
// {
//     ///id документа
//     doc_id: number,
//     checked?: DateUserInfo,
//     complex_information: ComplexDocumentInformation,
// }

// interface ComplexDocumentInformation
// {
//     ///id документа
//     doc_id: number,
//     ///наименование документа
//     doc_name: string,
//     ///расположение pdf образа документа
//     doc_image_path: string,
//     ///вид документа
//     type_name: string,
//     redaction: Redaction,
//     passing: Passing,
// }
// interface Redaction 
// {
//     ///создание редации
//     redaction_create?: DateUserInfo,
//     ///обновление редации
//     redaction_update?: DateUserInfo,
//     ///редакция готова
//     redaction_ready?: DateUserInfo,
// }


// interface Passing 
// {
//     ///дата подписания документа
//     pass_date: string,
//     ///номер документа
//     pass_number: string,
//     ///хэш документа
//     passhash64: string
// }

// type DateUserInfo = 
// {
//     ///дата
//     date: string,
//     ///id юзера
//     user_id: number,
// }

// Схема для DateUserInfo
const DateUserInfoSchema = z.object({
    date: date_time_schema, // coerce позволяет преобразовывать строки в Date
    user_id: z.number()
  });
  
  // Схема для Redaction
  const RedactionSchema = z.object({
    redaction_create: DateUserInfoSchema.nullable(),
    redaction_update: DateUserInfoSchema.nullable(),
    redaction_ready: DateUserInfoSchema.nullable()
  });
  
  // Схема для Passing
  const PassingSchema = z.object({
    pass_date: date_time_schema,
    pass_number: z.string(),
    passhash64: z.string()
  });
  
  // Схема для ComplexDocumentInformation
  const ComplexDocumentInformationSchema = z.object({
    doc_id: z.number(),
    doc_name: z.string(),
    doc_image_path: z.string(),
    type_name: z.string(),
    redaction: RedactionSchema,
    passing: PassingSchema
  });
  
  // Основная схема для Document
  const DocumentSchema = z.object({
    doc_id: z.number(),
    checked: DateUserInfoSchema.nullable(),
    complex_information: ComplexDocumentInformationSchema
  });
  const DocumentsSchema = z.array(DocumentSchema);

  type Documents = Document[];
  type DateUserInfo = z.infer<typeof DateUserInfoSchema>;
  type Redaction = z.infer<typeof RedactionSchema>;
  type Passing = z.infer<typeof PassingSchema>;
  type ComplexDocumentInformation = z.infer<typeof ComplexDocumentInformationSchema>;
  type Document = z.infer<typeof DocumentSchema>;

export type {DateUserInfo, Document, Documents, Passing, Redaction, ComplexDocumentInformation }
export {DocumentSchema, DocumentsSchema}