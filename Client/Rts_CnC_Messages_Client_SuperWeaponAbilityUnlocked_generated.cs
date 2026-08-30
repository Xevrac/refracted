using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_SuperWeaponAbilityUnlocked
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.SuperWeaponAbilityUnlocked); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.SuperWeaponAbilityUnlocked)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize SuperWeaponId
            s.Write(value.SuperWeaponId);
            //  Serialize AbilityId
            s.Write(value.AbilityId);
            //  Serialize MillisecondsToReenable
            s.Write(value.MillisecondsToReenable);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.SuperWeaponAbilityUnlocked)) as Rts.CnC.Messages.Client.SuperWeaponAbilityUnlocked;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize SuperWeaponId
            s.Read(out value.SuperWeaponId);
            //  Deserialize AbilityId
            s.Read(out value.AbilityId);
            //  Deserialize MillisecondsToReenable
            s.Read(out value.MillisecondsToReenable);

            return value;
        }
        
    }
}
