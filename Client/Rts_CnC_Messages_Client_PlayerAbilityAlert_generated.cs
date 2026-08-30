using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_PlayerAbilityAlert
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.PlayerAbilityAlert); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.PlayerAbilityAlert)obj;
            //  Serialize AbilityId
            s.Write(value.AbilityId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.PlayerAbilityAlert)) as Rts.CnC.Messages.Client.PlayerAbilityAlert;
            //  Deserialize AbilityId
            s.Read(out value.AbilityId);

            return value;
        }
        
    }
}
