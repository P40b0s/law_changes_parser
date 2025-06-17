import { NIcon } from "naive-ui";
import router from "../router";
import { Component, computed, type ComputedRef, h, VNode } from "vue";
import { RouteMeta } from "vue-router";
import { Role } from "../types/user_role";
import { Privilegy } from "../types/privilegy";
import {
  UserAvatar as ProfileIcon,
  ChartLineData as StatisticIcon,
  UserSettings as SettingsIcon,
  DocumentView as DocumentIcon,
  MailAll as PacketsIcon,
  Login as LoginIcon
} from '@vicons/carbon'
import Login from '@/views/Login.vue'
import Documents from '@/views/Documents.vue'
import Reports from "@/views/Reports.vue";
import Profile from "@/views/Profile.vue";
import Users from '@/views/Users.vue';
import useUser from "@/composables/useUser";
import { title } from 'process';
import MenuUserIcon from "@/components/MenuUserIcon.vue";
import Packets from "@/views/Packets.vue";
import useVisible from "@/composables/useVisible";

const {visible} = useVisible();
export enum RouteName
{
  Documents = "documents",
  Login = "login",
  Profile = "profile",
  Reports = "reports",
  Users = "users",
  Packets = "packets",
}


type MenuItem = {
  title: string,
  icon: Component,
  placement: 'up' | 'down'
}
class Routing
{
  
  name: RouteName;
  title: string;
  roles: Role[];
  privilegies: Privilegy[]
  reques_auth: boolean;
  path: string;
  component: VNode;
  menu?: MenuItem;
   /**
     * Отображает иконку в меню если текущая роль юзера подходит для этого маршрута
     * 
     */
  visible: ComputedRef<boolean>;

  constructor(name: RouteName, path: string, title: string, component: VNode)
  {
    this.name = name;
    this.title = title;
    this.roles = [];
    this.privilegies = [];
    this.reques_auth = false;
    this.path = path;
    this.component = component;
    this.visible = visible(this.roles, this.privilegies);
  }
   
  with_menu(title: string, icon: Component, placement: 'up' | 'down' = 'up'): Routing
  {
    this.menu = {
      title,
      icon,
      placement
    };
    return this;
  }
  with_roles(roles: Role[]): Routing
  {
    this.roles = roles;
    this.reques_auth = roles.length > 0;
    return this;
  }
  with_privilegies(privilegies: Privilegy[]): Routing
  {
    this.privilegies = privilegies;
    return this;
  }
  get_meta(): RouteMeta
  {
    return {
      	requiresAuth: this.reques_auth,
        title: this.title,
        roles: this.roles,
        privilegies: this.privilegies
    } as RouteMeta
  }
  get_path()
  {
    return this.path;
  }
  get_component()
  {
    return this.component;
  }
  get_name()
  {
    return this.name;
  }
  render_menu_label()
    {
        return h('div',
            {
                style:
                {
                    fontSize: "16px",
                    background: "transparent",
                    padding: "2px"
                },
                onClick:()=>
                {
                    router.push({name: this.name})
                }
            },
             this.menu?.title
        )
    }
    render_menu_icon() 
    {
        return () => h(NIcon, 
        {
          color: router.currentRoute.value.name == this.name ? 'rgb(146,230,26)' : 'rgb(139,140,115)',
          onClick:()=>
          {
            router.push({name: this.name})
          }
        }, 
        { 
            default: () => h(this.menu?.icon ?? 'span') 
        })
    }
  

    //menu_visible = computed(() => 
    //{
    //  if(!this.menu)
    //    return false;
      // const {get_role} = useUser();
      // if(this.roles.length == 0)
      //   return true
      // const current_role = get_role();
      // if(current_role == 'Administrator')
      //  return true;
      // if(this.roles.includes(current_role))
      //   return true;
      // else
      //   return false
    //})
}

const routes = new Map<RouteName, Routing>([
  
    [RouteName.Documents, new Routing(
      RouteName.Documents,
      '/documents',
      'Список документов',
      h(Documents))
      .with_menu('Список документов', DocumentIcon)
      .with_roles(['Administrator', 'User'])
    ],
    [RouteName.Login, new Routing(
      RouteName.Login,
      '/login',
      'Страница входа',
      h(Login)
    )],
    [RouteName.Profile, new Routing(
      RouteName.Profile,
      '/profile',
      'Профиль',
      h(Profile))
      .with_menu('Профиль', MenuUserIcon, 'down')
      .with_roles(['Administrator', 'User'])
    ],
    [RouteName.Reports, new Routing(
      RouteName.Reports,
      '/reports',
      'Отчеты',
      h(Reports))
      .with_menu('Отчеты', StatisticIcon)
      .with_roles(['Undefined']),
    ],
    [RouteName.Packets, new Routing(
      RouteName.Packets,
      '/packets',
      'Пакеты',
      h(Packets))
      .with_menu('Пакеты', PacketsIcon)
      .with_roles(['Administrator', 'User'])
      .with_privilegies(['FilesUpload']),
    ],
    [RouteName.Users, new Routing(
      RouteName.Users,
      '/users',
      'Список пользователей',
      h(Users))
      .with_menu('Список пользователей', SettingsIcon)
      .with_roles(['Administrator'])
    ],
  ]
)
export {routes};
