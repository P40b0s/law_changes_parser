<template lang="pug">
.profile(v-if="profile != null")
	n-radio-group.theme-buttons(v-if="props.theme_selector" v-model:value="theme" @update:value="change_theme" name="theme_group")
		n-radio-button(value="dark" label="Темная тема")
		n-radio-button(value="light" label="Светлая тема")
	.main
		n-card.left
			n-form(
				ref="formRef"
				:label-width="80"
				:model="profile"
				:rules="rules"
				size="medium")
					n-form-item(v-if="role == 'Administrator'" label="Идентификатор" path="id")
						n-input-number(v-model:value="user_id" :min="0" :max="255")
					n-form-item(label="Фамилия" path="surname")
						n-input(v-model:value="profile.surname" placeholder="Введите фамилию")
					n-form-item(label="Имя" path="first_name")
						n-input(v-model:value="profile.first_name" placeholder="Введите имя")
					n-form-item(label="Отчество" path="second_name")
						n-input(v-model:value="profile.second_name" placeholder="Введите отчество")
					n-form-item(label="Имя пользователя" path="username")
						n-input(v-model:value="profile.username" :disabled="role != 'Administrator'" placeholder="")
					n-form-item(v-if="element_is_visible" label="Пароль" path="password")
						n-input(v-model:value="password" placeholder="")
		n-card.right
			n-form-item(label="Аватар" path="avatar")
				label
					input(
					type="file" 
					accept="image/*" 
					style="display: none"
					@change="handleFileUpload")
					n-tooltip(v-if="previewUrl") Нажмите для загрузки фотографии аватара
						template(#trigger)
							img.ava-img(:src="previewUrl")
					img.ava-img(:src="HomerIcon" v-else)
			n-form-item(label="Роль" path="role")
				role-selector(v-model:value="profile.role" :disabled="element_is_disabled")
			n-form-item(label="Права" path="privilegies")
				privileges-editor(v-model:value="profile.privilegies" :disabled="element_is_disabled")
	n-button.save-button(:disabled="not_valid" @click="save" :type="not_valid ? 'error' : 'success'" @mouseover="mouse_over") Сохранить
	n-button.save-button(v-if="props.exit" @click="exit" type="error") Выйти из сессии
loader.ld(v-else status="Загрузка профиля...")
</template>
    
<script lang="ts">
import { ref, type Component, watch, inject, onMounted, onUnmounted, computed, onBeforeUnmount, h, toRefs, readonly } from 'vue';
import { type Events, type Emitter } from '../services/emitter';
import { NForm, NFormItem, NSelect, NInput, NRadioButton, NInputNumber, NRadioGroup, NButton, darkTheme, NDivider, NTooltip, NCard, type FormInst, type FormRules, type FormItemRule, type FormValidationError } from 'naive-ui';
import { notify_service } from '@/services/notification_service';
import { http_sevice } from '@/services/http_service';
import { type UserInfoUpdate, type UserInfo, type CreateUserPayload } from '@/types/user_info';
import Loader from '@components/Loader.vue'
import { compressImage } from '@/services/helpers';
import PrivilegesEditor from './PrivilegesEditor.vue';
import RoleSelector from './RoleSelector.vue';
import useUser from '@/composables/useUser';
import HomerIcon from '@svg/homer.svg'
import { type Role, roles } from '@/types/user_role';
import { type Theme, useTheme } from '@/composables/useTheme';
import { boolean } from 'zod';
import { useImage } from '@/composables/useImage';
import useVisible from '@/composables/useVisible';

</script>
<script lang="ts" setup>
const emitter = inject<Emitter<Events>>('emitter') as Emitter<Events>;
const props = defineProps<{
    profile?: UserInfo,
	theme_selector?: boolean,
	exit?: boolean,
	new_user?: boolean
}>()
const {visible, disabled} = useVisible();
const element_is_visible = visible(['Administrator', 'User']);
const element_is_disabled = disabled(['User']);
const emits = defineEmits<{
    (e: 'save', profile: UserInfoUpdate): void,
    (e: 'update:profile', profile: UserInfo): void,
	(e: 'update:new_user', b: boolean): void,
}>()
const formRef = ref<FormInst | null>(null)
const profile = ref<UserInfo|null>(null);
const not_valid = ref(false);
const selectedFile = ref<File|null>(null);
const previewUrl = ref<string|null>();
const image_to_upload = ref<Blob>()
const { get_role, exit } = useUser();
const { get_avatar, update_avatar, get_avatars, update_avatar_from_blob } = useImage();
const selected_role = ref();
const role = get_role();
const {light_theme, dark_theme, get_current_theme} = useTheme();
const theme = ref(get_current_theme().value);
const user_id = ref<number>(-1);
const password = ref<string>("");
const is_new = ref<boolean>(false);
const change_theme = (value: Theme) =>
{
	if(value == 'dark')
		dark_theme();
	if(value == 'light')
		light_theme();
}
// const validate_password = (rule: FormItemRule, value: string): boolean => 
// {
//     return profile.value != null && value === profile.value.
    
// }
const rules: FormRules =
{
    first_name: [
        {
            required: true,
            message: 'Необходимо ввести имя',
            trigger: ['input','blur', 'focus'],
        }
    ],
    second_name: [
        {
            required: true,
            message: 'Необходимо ввести отчество',
            trigger: ['input','blur', 'focus'],
        }
    ],
    surname: [
        {
            required: true,
            message: 'Необходимо ввести фамилию',
            trigger: ['input','blur', 'focus'],
        }
    ],
	
    // reentered_password: [
    //     {
    //       required: true,
    //       message: 'Для аодтверждения введите пароль повторно',
    //       trigger: ['input', 'blur']
    //     },
    //     {
    //       validator: validate_password,
    //       message: 'Password is not same as re-entered password!',
    //       trigger: ['blur', 'password-input']
    //     },
    //   ]
}
const mouse_over = (e: MouseEvent) =>
{
    e.preventDefault()
	const id = user_id.value >=0 && user_id.value <= 255;
    formRef.value?.validate(
        (errors: Array<FormValidationError> | undefined) => 
        {
        if (!errors && id) 
        {
            not_valid.value = false;
        }
        else 
        {
            not_valid.value = true;
        }
        }
    )
}
watch(() => props.profile, async (n) =>
{
    if(n)
	{
		
		profile.value = n;
        user_id.value = n.id;
		password.value = "";
		if(props.new_user)
		{
			is_new.value = props.new_user;
		}
		previewUrl.value = await get_avatar(n.id) as string;
	}
}, 
{immediate: true})
onUnmounted(()=>
{
	if(previewUrl.value)
		URL.revokeObjectURL(previewUrl.value)
})
const handleFileUpload = async (event: Event) => 
{
    const target = event.target as HTMLInputElement;
    const file = target.files?.item(0);
    if (!file) return;
    selectedFile.value = file as File;
    image_to_upload.value = await compressImage(selectedFile.value as File, {quality: 0.5, maxHeight: 200, maxWidth: 200, mimeType: 'image/webp'})
    // Создаем превью
    previewUrl.value =  URL.createObjectURL(image_to_upload.value);
};

const save = async () => 
{
	console.log(is_new.value );
    if (!selectedFile.value && !profile.value && !image_to_upload.value) return;
    if(is_new.value)
	{
		const user = 
		{
			id: user_id.value,
			username: profile.value?.username,
			first_name: profile.value?.first_name,
			second_name: profile.value?.second_name,
			password: password.value.length == 0 ? undefined : password.value,
			surname: profile.value?.surname,
			role: profile.value?.role,
			privilegies: profile.value?.privilegies
		} as CreateUserPayload;
		const formData = new FormData();
		if(image_to_upload.value)
		{
			formData.append('avatar', image_to_upload.value);
			update_avatar_from_blob(user_id.value, image_to_upload.value);
		}
		formData.append("user_info", JSON.stringify(user));
		const result = await http_sevice.create_user(formData, profile.value?.username as string);
		if(result)
		{
			//выставляем обратно на false
			is_new.value = false;
			emits('update:new_user', false);
		}
	}
	else
	{
		const user = 
		{
			id: profile.value?.id,
			first_name: profile.value?.first_name,
			second_name: profile.value?.second_name,
			surname: profile.value?.surname,
			password: password.value.length == 0 ? undefined : password.value,
			role: profile.value?.role,
			privilegies: profile.value?.privilegies
		} as UserInfoUpdate;
		const formData = new FormData();
		if(image_to_upload.value)
		{
			formData.append('avatar', image_to_upload.value);
			update_avatar_from_blob(user_id.value, image_to_upload.value);
		}
		if(user_id.value != profile.value?.id)
		{
			formData.append("new_user_id", user_id.value.toString());
		}
		formData.append("username", profile.value?.username as string);
		formData.append("user_info", JSON.stringify(user));
		await http_sevice.profile_update(formData, profile.value?.username as string);
	}
};
</script>
    
<style lang="scss" scoped>
$height: 700px;
.profile
{
    display: flex;
    flex-direction: column;
    align-items: center;
    overflow: 'hidden';
    overflow-y: 'hidden';
    
}
.ld
{
    width: 100%;
    margin-top: -5px;
}
.main
{
    display: flex;
    flex-direction: row;
}
.left
{
    display: flex;
    flex-direction: column;
    width: 300px;
    height: $height;
}
.right
{
    display: flex;
    flex-direction: column;
    width: 300px;
    height: $height;
}
.save-button
{
    width: 200px;
	align-self: center;
    margin-top: 10px;
}
.theme-buttons
{
    margin-bottom: 10px;
}
.ava-img
{
    cursor: pointer;
	max-width: 200px;
}
</style>