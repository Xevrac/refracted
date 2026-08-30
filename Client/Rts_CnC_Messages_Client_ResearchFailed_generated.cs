using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_ResearchFailed
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.ResearchFailed); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.ResearchFailed)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize UpgradeType
            s.Write(value.UpgradeType);
            //  Serialize IsGlobalUpgrade
            s.Write(value.IsGlobalUpgrade);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.ResearchFailed)) as Rts.CnC.Messages.Client.ResearchFailed;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize UpgradeType
            s.Read(out value.UpgradeType);
            //  Deserialize IsGlobalUpgrade
            s.Read(out value.IsGlobalUpgrade);

            return value;
        }
        
    }
}
