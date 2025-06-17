import { z } from 'zod';
import { RoleSchema } from "./user_role";
import { PrivilegySchema } from "./privilegy";

export const avatar_parse_schema = z.custom<Uint8Array|undefined>()
.transform((val) => {
  if(val)
    new Uint8Array(val)
  else
    val
});
const UsernameSchema = z.object({
  username: z.string(),
});
const AvatarSchema = z.object({
  avatar: avatar_parse_schema,
});
const PasswordSchema = z.object({
  password: z.string().nullish(),
});
const TokenSchema = z.object({
  token: z.string().nullish(),
});
const UserInfoBaseSchema = z.object({
  id: z.number().int().min(0).max(255), // u8 в Rust - число от 0 до 255
  first_name: z.string(),
  second_name: z.string(),
  surname: z.string(),
  role: RoleSchema,
  privilegies: z.array(PrivilegySchema)
});
// Основная схема для UserInfo
const UserInfoSchema = UserInfoBaseSchema
  .merge(TokenSchema)
  .merge(UsernameSchema) //z.object({
//   id: z.number().int().min(0).max(255), // u8 в Rust - число от 0 до 255
//   //username: z.string(),
//   first_name: z.string(),
//   second_name: z.string(),
//   surname: z.string(),
//   //avatar: z.union([z.instanceof(Uint8Array), z.number().array()]).nullish(),
//   token: z.string().nullish(),
//   role: RoleSchema,
//   privilegies: z.array(PrivilegySchema)
// });


// Основная схема для UserInfo
const UserInfoUpdateSchema = UserInfoBaseSchema
  .merge(AvatarSchema)
  .merge(PasswordSchema);

const CreateUserPayloadSchema = UserInfoBaseSchema
  .merge(UsernameSchema)
  .merge(AvatarSchema)
  .merge(PasswordSchema);

type UserInfo = z.infer<typeof UserInfoSchema>;
type UserInfoUpdate = z.infer<typeof UserInfoUpdateSchema>;
type CreateUserPayload = z.infer<typeof CreateUserPayloadSchema>;
export {type UserInfo, UserInfoSchema, type UserInfoUpdate, UserInfoUpdateSchema, type CreateUserPayload , CreateUserPayloadSchema}

