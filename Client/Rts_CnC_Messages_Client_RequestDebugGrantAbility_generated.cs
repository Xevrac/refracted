using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestDebugGrantAbility
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestDebugGrantAbility); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestDebugGrantAbility)obj;
            //  Serialize AbilityHash
            s.Write(value.AbilityHash);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestDebugGrantAbility)) as Rts.CnC.Messages.Client.RequestDebugGrantAbility;
            //  Deserialize AbilityHash
            s.Read(out value.AbilityHash);

            return value;
        }
        
    }
}
