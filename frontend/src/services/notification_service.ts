import { NButton } from 'naive-ui';
import { h } from 'vue';
import { useVtEvents, useToast } from 'vue-toastify';



// useVtEvents().once('vtPaused', payload => {
//     if (payload.id === toast.id) {
//         // do something
//     }
// })
export const notify_warning = (title: string, body: string) =>
{
    const t =  useToast().warning(body, title);
    console.log(t);
    //для примера можно сделать свои теплейты для уведомлений
    //const toast = useToast().authenticationError({body: "БОДИ", title: "TITLE", type: 'error'})
}
class NotificationService
{
    notify_warning(title: string, body: string)
    {
        const t =  useToast().warning(body, title);
        console.log(t);
        //для примера можно сделать свои теплейты для уведомлений
        //const toast = useToast().authenticationError({body: "БОДИ", title: "TITLE", type: 'error'})
    }
    notify_error(title: string, body: string)
    {
        const t =  useToast().error(body, title);
        console.log(t);
        //для примера можно сделать свои теплейты для уведомлений
        //const toast = useToast().authenticationError({body: "БОДИ", title: "TITLE", type: 'error'})
    }
    notify_success(title: string, body: string)
    {
        const t =  useToast().success(body, title);
        console.log(t);
        //для примера можно сделать свои теплейты для уведомлений
        //const toast = useToast().authenticationError({body: "БОДИ", title: "TITLE", type: 'error'})
    }
    notify_test()
    {
        const t1 = h(NButton, {class: "123"}, {default:() => "123123123123"});
        const t =  useToast().warning(t1, "BUTTON!");
        console.log(t);
        //для примера можно сделать свои теплейты для уведомлений
        //const toast = useToast().authenticationError({body: "БОДИ", title: "TITLE", type: 'error'})
    }
}

const notify_service = new NotificationService();
export {notify_service}