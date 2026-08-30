using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_ResearchComplete
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.ResearchComplete); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.ResearchComplete)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize Upgrade
            s.Write(value.Upgrade);
            //  Serialize IsGlobalUpgrade
            s.Write(value.IsGlobalUpgrade);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.ResearchComplete)) as Rts.CnC.Messages.Client.ResearchComplete;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize Upgrade
            s.Read(out value.Upgrade);
            //  Deserialize IsGlobalUpgrade
            s.Read(out value.IsGlobalUpgrade);

            return value;
        }
        
    }
}
