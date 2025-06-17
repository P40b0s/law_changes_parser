import { computed, ref, watch } from "vue";
import useUser from "./useUser";
import { type Role } from "@/types/user_role";
import { type Privilegy } from '@/types/privilegy';


export default function useVisible()
{
    const {get_user} = useUser();
    const visible = (roles: Role[], privilegies: Privilegy[] = []) =>
    {
        
        return computed(() =>
        {
            const user = get_user();
            if(user.value !== null)
            {
                const role_granted = roles.length == 0 ? true : roles.includes(user.value.role);
                const privilegies_granted = privilegies.length == 0 ? true : privilegies.some(p=>
                {
                    return user.value?.privilegies.includes(p)
                })
                console.log("permissions: ", role_granted, privilegies_granted);
                return role_granted && privilegies_granted
            }
            console.log("permissions: rejected");
            return false;
        })
    }
    const disabled = (roles: Role[], privilegies: Privilegy[] = []) =>
    {
        return visible(roles, privilegies);
    }

    return {visible, disabled}
    
}