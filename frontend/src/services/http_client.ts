import { createErr, type Result, createOk, type Err } from 'option-t/plain_result';
//import { notification_service } from "./notification";
import { match } from 'ts-pattern';
import useUser from '@/composables/useUser';
import { z, ZodSchema, ZodType } from 'zod';

export const api_path = import.meta.env.PUBLIC_API_ADDRESSE + ":" + import.meta.env.PUBLIC_API_PORT + "/";
class HTTPError extends Error {}
HTTPError.prototype.name = "HTTPError";
const { get_token } = useUser();

const gen_req_error = async (resp: Response): Promise<Err<HTTPError>> => 
{
    const err = await match(resp.status)
    .with(406, async () => 
    {
        const text = await resp.text();
        return new HTTPError(`${text}`)
    })
    .otherwise(async () => new HTTPError(`Ошибка получения данных от АПИ: ${resp.status}:${resp.statusText}`))
    //200-299 status code
    console.error(err);
    return createErr(err);
}
const fetch_error = (error: unknown): Err<HTTPError> => 
{
    const err = new HTTPError(`Ошибка запроса: ${error}`);
    console.error(err);
    return createErr(err);
}

const get = async <Q extends string, R, S extends ZodType>(path_and_query: Q, auth: boolean, schema?: S): Promise<Result<R, HTTPError>> => 
{
    try
    {
        let headers = new Headers();
        if (auth)
        {
            headers.append("Authorization",  "Bearer " + get_token());
        }
        const response = await fetch(api_path + path_and_query, 
        {
            method: 'GET',
            credentials: 'include',
            headers: headers
            
        });
        if (!response.ok) 
        {
            return await gen_req_error(response);
        }
        else
        {
            return get_body(response, schema);
        }
    }
    catch (error)
    {
        return fetch_error(error);
    }
}

const get_body = <B, S extends ZodType>(response: Response, schema?: S): Promise<Result<B, HTTPError>> =>
{
    const content_type = response.headers.get('Content-Type');
    const def = async () => { return createOk({} as B);}
    if(content_type)
    {
        const object = match(content_type)
        .with('application/json', async () =>
        {
            if(schema)
            {
                const json = await response.json();
                try
                {
                    const parsedDocument: B = schema.parse(json);
                    return createOk(parsedDocument);
                }
                catch(error)
                {
                    if (error instanceof z.ZodError) 
                    {
                        // Полный вывод ошибки в формате JSON
                        console.error(JSON.stringify(error, null, 2));   
                    }
                    return createErr(new HTTPError(error as string));
                }
            }
            else
            {
                const json: B = await response.json();
                return createOk(json);
            }
        })
        .with('text/plain; charset=utf-8', async () =>
        {
            return createOk(await response.text() as B);
        })
        .with('multipart/form-data', async () =>
        {
            return createOk(await response.blob() as B);
        })
        .otherwise(async () => createErr(new HTTPError("Необработанный тип сообщения " + content_type + " -> " + await response.text())));
        return object;
    }
    else
    {
        return def();
    }
}
// (type === 'object') {
//     if (Object.keys(v).length < 1) {
//         return true;
/**
 * const tt: Result<string, HTTPError> = await post("", 'PATCH')
 */
const post = async <Q extends string, P, R extends unknown, S extends ZodSchema, H extends Record<string, string>>(path_and_query: Q, method: "PATCH"|"POST", auth: boolean, payload?: P, schema?: S, headers?: H): Promise<Result<R, HTTPError>> => 
{
    try
    {
        let headers = new Headers();
        if (auth)
        {
            headers.append("Authorization",  "Bearer " + get_token());
        }
        if(headers)
        {
            Object.entries(headers).forEach(h=>
            {
                headers.append(h[0], h[1]);
            })
        }
        if(payload)
        {
            let is_form_data = isFormData(payload);
            if(!is_form_data)
                headers.append('content-type', 'application/json')
            const body = is_form_data ? (payload as unknown) as FormData : JSON.stringify(payload);
            const response = await fetch(api_path + path_and_query, 
            {
                method: method,
                body: body,
                credentials: 'include',
                headers: headers
            });
            if (!response.ok) 
            {
                return await gen_req_error(response);
            }
            else
            {
                return get_body(response, schema);
            }
        }
        else
        {
            const response = await fetch(api_path + path_and_query, 
            {
                method: method,
                credentials: 'include',
                headers: headers
            });
            if (!response.ok) 
            {
                return await gen_req_error(response);
            }
            else
            {
                return createOk({} as R);
            }
        }
    }
    catch (error)
    {
        return fetch_error(error);
    }
}

function isFormData(obj: unknown): obj is FormData 
{
    return obj instanceof FormData;
}
export {post, get, HTTPError}