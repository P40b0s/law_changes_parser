import 
{
    h,
    defineComponent,
    defineAsyncComponent,
    inject,
    onMounted,
    type CSSProperties,
    ref,
    type PropType,
    nextTick,
    getCurrentInstance,
    toRef,
    onUnmounted,
    onBeforeMount,
    onBeforeUnmount,
    type Component,
    computed
  } from 'vue'
//import '../assets/styles/scrollable_table.scss'
import { NAvatar, NBadge, NButton, NCard, NDivider, NForm, NFormItem, NIcon, NInput, NList, NListItem, NModal, NPagination, NScrollbar, NSpin, NTab, NTabPane, NTable, NTabs, NTbody, NThead, NTooltip, NTr } from 'naive-ui';
import { DateFormat, DateTime} from '../../services/date.ts';
import {LiveSearch} from '../live_search.tsx'
import { match } from 'ts-pattern';
import { add_user_ico, chain_ico, health_ico, homer_ico, info_ico, law_ico, location_ico, palm_ico, time_ico, user_ico } from '@services/svg.ts';
import {DocumentBlank, Inspection, PageNumber, Rewind5, Time, Barcode, UserAvatarFilledAlt, RequestQuote, InformationSquareFilled, CheckboxChecked, InformationSquare} from '@vicons/carbon'
import "./document_list.scss"
import ChainIcon from './ChainIcon.vue';
import router from '../../router.ts';
import {type Documents, type Document} from '@/types/document.ts';
import { http_sevice } from '@services/http_service.ts';
import { UserInfo } from '@/types/user_info.ts';
import UserIcon from '../UserIcon.vue';
import ActivityButtons from './ActivityButtons.vue';
import StatusesList from './StatusesList.vue';
import Loader from '@components/Loader.vue'
import DynamicRender from '../DynamicRender.vue';
import Editor from '../Editor.vue';
import { type Emitter, type Events } from '@/services/emitter.ts';
import ComparatorModal from './ComparatorModal.vue';

const search_value = ref("");
export const DocumentsList = defineComponent({
    setup (props, { slots }) 
    {
        const emitter = inject<Emitter<Events>>('emitter') as Emitter<Events>;
        const current_page = ref(1);
        const limit = 50;
        const overall_count = ref(0);
        const offset = computed(()=> (current_page.value -1) * limit);
        const documents = ref<Documents>([]);
        const is_loading = ref(false);
        const show_text = ref(false);
        const selected_document_id = ref(0);
        const documents_updated_event = async (refs: Document) =>
        {
            //await get_documents((current_page.value -1) * limit)
            // let doc = references.value.find(f=>f.id == refs.id);
            // if(doc)
            // {
            //     doc = refs.refs;
            // }
        }

        onUnmounted(()=>
        {
            //emitter.off('references_updated', reference_updated_event);
        })
        
        const complex = () =>
        {
            return h('div',
                {
                    style:
                    {
                        //height: 'calc(100vh - 20px)', // 60px - примерное пространство для пагинации и кнопок
                        overflow: 'hidden',
                        overflowY: 'hidden',
                        
                    } as CSSProperties
                },
            [
                h(NModal,
                {
                    show: show_text.value
                },
                {
                    default:() => h(NCard,
                        {

                        },
                        {
                           default:() => h(ComparatorModal, {doc_id: selected_document_id.value})
                           //default:() => h(Editor, {doc_id: selected_document_id.value})
                        }
                    )
                }
                ),
                h(ActivityButtons, 
                {
                    documents: documents.value as Documents,
                    count: overall_count.value,
                    offset: offset.value,
                    limit: limit,
                    is_loading: is_loading.value,
                    'onUpdate:documents':(d) => documents.value = d,
                    'onUpdate:count':(c) => overall_count.value = c,
                    'onUpdate:is_loading':(l) => is_loading.value = l,
                    'onUpdate:offset': () => current_page.value = 1
                }),
                is_loading.value ? h(Loader, 
                {
                    status:"Загрузка",
                    style:
                    {
                        height: '100%'
                    } as CSSProperties
                }) : h('div', 
                    [
                        h(list),
                        h(NPagination,
                        {
                            size: 'large',
                            itemCount: overall_count.value,
                            pageSizes: [limit],
                            showSizePicker: true,
                            simple: false,
                            page: current_page.value,
                            onUpdatePage: async (page) => 
                            {
                                current_page.value = page;
                                //await get_documents((page-1) * limit);
                            },
                        })
                    ])
            ])
        }

        const list = () =>
        {
            return  h(NList,
            {
                bordered: false,
            },
            {  
                default:() => h(NScrollbar,
                {
                    style:{
                        maxHeight: '90vmin',
                        paddingRight: "5px",
                        marginTop:'5px',
                        marginBottom:'5px'
                    } as CSSProperties
                },
                {
                    default:() => documents.value.map(d=>
                    {
                        return item(d as Document)
                    })
                })
            })
        }

        const item = (d: Document) =>
        {
            return h(NListItem,
                {
                    class:"main-n-list-item",
                    onClick:() =>
                    {
                        selected_document_id.value = d.doc_id;
                        show_text.value = true;
                        //emitter.emit('open_pdf', d.doc_id);
                    }
                },
                {
                    default:() => list_item_body(d),
                })
        }


        const right_icon_component = (d: Document, value: string|number, tooltip: string, color: string, icon: Component) =>
        {
            return h(NTooltip,
                {
                    trigger: 'hover'
                },
                {
                    trigger:()=> value.toString().length > 0 ?
                    h('div',
                        {
                            style:
                            {
                                display: 'flex',
                                alignItems: 'center',
                            } as CSSProperties
                        },
                        [
                            h(NIcon,
                            {
                                size: 25,
                                color: color,
                                component: icon,
                                style:
                                {
                                    marginRight: '4px',
                                }
                            }),
                            h('span', 
                            {
                                style:
                                {
                                    fontSize: '16px',
                                    fontWeight: 600,
                                } as CSSProperties
                            }, value),
                            h(NDivider, {vertical: true}),
                        ]) : h('span'),
                    default:() => tooltip,
                }
            )
        }
       

        // const icon_with_name = (value: string|number, tooltip: string, color: string, icon: Component) =>
        // {
        //     return h(NTooltip,
        //         {
        //             trigger: 'hover'
        //         },
        //         {
        //             trigger:()=>
        //             value.toString().length > 0 ? h('div',
        //                 {
        //                     style:
        //                     {
        //                         display: 'flex',
        //                         alignItems: 'center',
        //                     } as CSSProperties
        //                 },
        //                 [
        //                     h(NIcon,
        //                     {
        //                         size: 25,
        //                         color: color,
        //                         component: icon,
        //                         style:
        //                         {
        //                             marginRight: '4px',
        //                         }
        //                     }),
        //                     h('span', 
        //                     {
        //                         style:
        //                         {
        //                             fontSize: '16px',
        //                             textAlign: 'left'
        //                         } as CSSProperties
        //                     }, value),
        //                 ]) : h('span'),
        //             default:() => tooltip,
        //         }
        //     )
        // }

        const list_item_body = (d: Document) =>
        {
            return h('div',
                {
                    style:
                    {
                        display: 'flex',
                        flexDirection:'column',
                        alignItems:'flex-start',
                        cursor: 'pointer',
                    } as CSSProperties,
                    onClick:()=>
                    {
                        //router.push({name: 'diff', query: { id: r.id, redaction_id: r.redaction?.redaction_id, doc_hash: r.doc_hash, publication_id: r.publication_doc_card_id}})
                        //emitter.emit('open_comparator_window', r);
                    }
                },

                    [
                        h('div',
                        {
                            style:
                            {
                                fontWeight: '600',
                                borderBottom: "1px solid rgba(27, 126, 110, 0.85)",
                                marginRight: '12px',
                                display: 'flex',
                                alignItems: 'center',
                                justifyItems: 'center',
                                width: "99.5%",
                                backgroundColor: "var(--selection-color)",
                                padding: "2px",
                                background: "rgba( 74, 144, 226, 0.4 )",
                                boxShadow: "0 8px 32px 0 rgba( 31, 38, 135, 0.37 )",
                                backdropFilter: "blur( 8px )",
                                "-webkit-backdrop-filter": "blur( 8px )",
                                borderRadius: "10px",
                                gap: '5px',
                                border: "1px solid rgba( 255, 255, 255, 0.18 )"
                                
                            } as CSSProperties
                        },
                        {
                            default:()=>
                                [
                                    h(StatusesList, {redaction: d.complex_information.redaction, checked: d.checked}),
                                    h('div', 
                                        {
                                            style: 
                                            {
                                                fontSize: '18px'
                                            } as CSSProperties
                                        },
                                        complex_name(d)),
                                    h('div', 
                                    {
                                        style:
                                        {   
                                            display: 'flex',
                                            flexDirection: 'row-reverse',
                                            flexGrow: 2
                                        } as CSSProperties
                                    },
                                    d.checked ?
                                    [
                                        h(UserIcon, { user_id:  d.checked?.user_id}),
                                        right_icon_component(d, d.checked?.date.to_string(DateFormat.DateTime) ?? 0,  "Дата проверки", "#17bcde", CheckboxChecked)
                                    ] : [])

                                ]
                        }),
                        h('div',
                            {
                                style:
                                {
                                   display: 'flex',
                                   justifyContent: 'space-between',
                                   width: "99.5%",
                                   background: "rgba(75, 145, 226, 0.12)",
                                   boxShadow: "0 8px 32px 0 rgba( 31, 38, 135, 0.37 )",
                                   backdropFilter: "blur( 8px )",
                                   "-webkit-backdrop-filter": "blur( 8px )",
                                   borderRadius: "10px",
                                   marginTop:'5px',
                                   border: "1px solid rgba( 255, 255, 255, 0.18 )"
                                } as CSSProperties
                            },
                            [
                                h('div',
                                    {
                                        style:
                                        {
                                           display: 'flex',
                                           alignItems: 'stretch',
                                           flexDirection: 'column',
                                           height: '100%'
                                        } as CSSProperties
                                    },
                                    h('div',
                                        {
                                            style:
                                            {
                                                display: 'flex',
                                                alignItems: 'center',
                                            } as CSSProperties
                                        },
                                        [
                                            h(NIcon,
                                            {
                                                size: 30,
                                                color: "#29d129",
                                                component: RequestQuote,
                                                style:
                                                {
                                                    marginRight: '4px',
                                                }
                                            }),
                                            h('span', 
                                            {
                                                style:
                                                {
                                                    fontSize: '18px',
                                                    textAlign: 'left'
                                                } as CSSProperties
                                            }, d.complex_information.doc_name),
                                        ])
                                    
                                ),
                               
                           ]
                        )
                    ]
            )
        }
        const complex_name = (doc: Document) =>
        {
            return `${doc.complex_information.type_name} № ${doc.complex_information.passing.pass_number} от ${doc.complex_information.passing.pass_date.to_string(DateFormat.DotDate)}`
        }
        
        return {complex}
    },

   
    render ()
    {
        return h(this.complex)
    }
})