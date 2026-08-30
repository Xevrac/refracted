using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_UnitAbilityBlocked
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.UnitAbilityBlocked); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.UnitAbilityBlocked)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize UnitAbility
            s.Write(value.UnitAbility);
            //  Serialize Blocked
            s.Write(value.Blocked);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.UnitAbilityBlocked)) as Rts.CnC.Messages.Client.UnitAbilityBlocked;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize UnitAbility
            s.Read(out value.UnitAbility);
            //  Deserialize Blocked
            s.Read(out value.Blocked);

            return value;
        }
        
    }
}
