import router from "@/router";
import emitter from "@/services/emitter";
import { base64_to_uint8_array } from "@/services/helpers";
import { http_sevice } from "@/services/http_service";
import { notify_service } from "@/services/notification_service";
import { Privilegy } from "@/types/privilegy";
import { UserInfo } from "@/types/user_info";
import { Role } from "@/types/user_role";
import { match } from "ts-pattern";
import { computed, readonly, ref, Ref } from "vue";

const user: Ref<UserInfo| null> = ref(null)
export const is_authentificated = computed(()=> !!user.value && !!user.value.token)
export default function useUser()
{
    const set_user = (new_user: UserInfo) => 
    {
        localStorage.setItem("user", JSON.stringify(new_user));
        user.value = new_user;
    }
    const load_user = (): boolean => 
    {
        const u = localStorage.getItem("user");
        if(u)
        {
            user.value = JSON.parse(u)
            if (user.value)
                return true;
            else
                return false;
        }
        else
        {
            notify_service.notify_warning("Юзер не найден", "Для работы вам необходимо войти в систему")
            return false;
        }
    }
    const get_role = (): Role => 
    {
        return user.value?.role ?? 'Undefined'
    }
    const get_role_name = (r: Role) =>
    {
        
        return match(r)
        .with('Administrator', () => "Администратор")
        .with('User', () => "Юзер")
        .otherwise(() => "Неизвестно")
    }
    const get_privilegy_description = (p: Privilegy) =>
    {
        return match(p)
        .with('Check', () => "Проверка документов")
        .with('Delete', () => "Удаление документов")
        .with('Export', ()=> "Экспортирование порций")
        .with('Import', () => "Импортирование порций")
        .with('UsersList', () => "Доступ к списку юзеров")
        .with('FilesUpload', () => "Загрузка файлов новых документов")
        .exhaustive()
    }
    const get_privilegies = (): Privilegy[] => 
    {
        return user.value?.privilegies ?? [];
    }
    const exit = () => 
    {
        localStorage.removeItem("user");
        user.value = null;
        router.push({name: 'login'});
    }
    const get_user = () => readonly(user);
    const get_token = (): string | undefined | null => 
    {
        return user.value?.token;
    }
    const remove_token = () => 
    {
        if(user.value)
        {
            user.value.token = undefined;
            localStorage.setItem("user", JSON.stringify(user.value));
        }
    }
    return {set_user, load_user, get_role, get_privilegies, exit, get_user, get_token, get_privilegy_description, get_role_name}
}