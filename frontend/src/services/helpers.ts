import {  h, type RendererElement, type RendererNode, type VNode, type Ref } from 'vue';
import { DateFormat, DateTime } from './date';

const sleepNow = (delay: number) => new Promise((resolve) => setTimeout(resolve, delay))
const timer  =  (delay: number) =>  setTimeout(()  => { }, delay);

let tm : NodeJS.Timeout = setTimeout(()  => { }, 200); // eslint-disable-line

const groupByArray = (xs: any, key: any) => 
{ 
  return xs.reduce(function (rv: any, x: any) 
  { 
    const v: any = key instanceof Function ? key(x) : x[key];
    const el = rv.find((r:any) => r && r.key === v);
    if (el) 
    {
      el.values.push(x);
    } 
    else 
    { 
      rv.push({ key: v, values: [x] });
    } 
    return rv; 
  }, []); 
}
const group = <T, K extends keyof any>(list: T[], getKey: (item: T) => K) =>
list.reduce((previous, currentItem) => 
{
    const group = getKey(currentItem);
    if (!previous[group]) previous[group] = [];
    previous[group].push(currentItem);
    return previous;
}, {} as Record<K, T[]>);

const groupBy = (list: any, keyGetter: any) =>
{
    const map = new Map();
    list.forEach((item: any) => {
         const key = keyGetter(item);
         const collection = map.get(key);
         if (!collection) {
             map.set(key, [item]);
         } else {
             collection.push(item);
         }
    });
    return map;
}
export const date_str = (dt: string|null|undefined, format: DateFormat) =>
{
    if (dt == null || dt == undefined)
        return ""
    else
    {
        const d = DateTime.parse(dt);
        const date = d.to_string(format);
        return date;
    }
}
export const base64_to_uint8_array = (base64: string) =>  
{
  const binary = atob(base64);
  const len = binary.length;
  const bytes = new Uint8Array(len);
  for (let i = 0; i < len; i++)
      bytes[i] = binary.charCodeAt(i);
  return bytes;
}

export const component_visible = < T extends VNode<RendererNode, RendererElement, {[key: string]: any;}> >(vis: boolean, f:() => T) =>
{
  if (vis)
      return f();
  else return h('span')
}


  function isError (e: unknown)  : [string, string]
    {
      if(e instanceof Error)
      {
        console.log(e.name);
        return [e.name, e.message];
      }
      return ["", ""];
    }

    function deep_clone<T>(obj: T): T
    {
      return JSON.parse(JSON.stringify(obj)) as T;
    }

export interface CompressImageOptions 
{
  quality?: number // Качество изображения (0-1)
  maxWidth?: number // Максимальная ширина
  maxHeight?: number // Максимальная высота
  mimeType?: string // Тип выходного файла ('image/jpeg', 'image/png' и т.д.)
}

/**
 * Сжимает изображение с возможностью изменения размера
 * @param file Входной файл изображения
 * @param options Параметры сжатия
 * @returns Promise с Blob сжатого изображения
 */
export const compressImage = async (
  file: File,
  options: CompressImageOptions = {}
): Promise<Blob> => 
{
  const {
    quality = 0.8,
    maxWidth = 2048,
    maxHeight = 2048,
    mimeType = 'image/jpeg'
  } = options

  try 
  {
    const bitmap = await createImageBitmap(file)
    
    // Рассчитываем новые размеры с сохранением пропорций
    let width = bitmap.width
    let height = bitmap.height
    
    if (width > maxWidth || height > maxHeight) 
    {
      const ratio = Math.min(maxWidth / width, maxHeight / height)
      width = Math.floor(width * ratio)
      height = Math.floor(height * ratio)
    }

    const canvas = new OffscreenCanvas(width, height)
    const ctx = canvas.getContext('2d')
    
    if (!ctx) 
    {
      throw new Error('Could not get canvas context')
    }

    // Отрисовка с масштабированием
    ctx.drawImage(bitmap, 0, 0, width, height)
    
    // Конвертация в Blob с указанными параметрами
    const blob = await canvas.convertToBlob({
      type: mimeType,
      quality: Math.max(0, Math.min(1, quality)) // Обеспечиваем корректный диапазон
    })

    // Освобождаем ресурсы
    bitmap.close()
    
    return blob
  } 
  catch (error) 
  {
    console.error('Image compression failed:', error)
    // В случае ошибки возвращаем оригинальный файл
    return file
  }
}



export {sleepNow, timer, isError, groupBy, deep_clone, group}