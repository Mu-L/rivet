/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { Email } from "../../../../common/types/Email";

export const EmailLinkedAccount: core.serialization.ObjectSchema<
    serializers.identity.EmailLinkedAccount.Raw,
    Rivet.identity.EmailLinkedAccount
> = core.serialization.object({
    email: Email,
});

export declare namespace EmailLinkedAccount {
    export interface Raw {
        email: Email.Raw;
    }
}
