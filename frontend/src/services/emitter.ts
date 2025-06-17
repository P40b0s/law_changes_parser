//import mitt, {type Emitter} from 'mitt'
import { Emitter } from 'strict-event-emitter'
const emitter = new Emitter<Events>()
export default emitter
export {type Events, type Emitter};


type Events =
{
    test: [string]
    update_profile: []
    delete_packet:[string]
    open_pdf: [number]
};