using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_SuperWeaponAbilityRemoved
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.SuperWeaponAbilityRemoved); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.SuperWeaponAbilityRemoved)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize SuperWeaponId
            s.Write(value.SuperWeaponId);
            //  Serialize AbilityId
            s.Write(value.AbilityId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.SuperWeaponAbilityRemoved)) as Rts.CnC.Messages.Client.SuperWeaponAbilityRemoved;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize SuperWeaponId
            s.Read(out value.SuperWeaponId);
            //  Deserialize AbilityId
            s.Read(out value.AbilityId);

            return value;
        }
        
    }
}
