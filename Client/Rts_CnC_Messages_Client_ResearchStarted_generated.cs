using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_ResearchStarted
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.ResearchStarted); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.ResearchStarted)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize FactoryEntityId
            s.Write(value.FactoryEntityId);
            //  Serialize UpgradeType
            s.Write(value.UpgradeType);
            //  Serialize ResearchTime
            s.Write(value.ResearchTime);
            //  Serialize LowPowerPenaltyResearchTime
            s.Write(value.LowPowerPenaltyResearchTime);
            //  Serialize IsGlobalUpgrade
            s.Write(value.IsGlobalUpgrade);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.ResearchStarted)) as Rts.CnC.Messages.Client.ResearchStarted;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize FactoryEntityId
            s.Read(out value.FactoryEntityId);
            //  Deserialize UpgradeType
            s.Read(out value.UpgradeType);
            //  Deserialize ResearchTime
            s.Read(out value.ResearchTime);
            //  Deserialize LowPowerPenaltyResearchTime
            s.Read(out value.LowPowerPenaltyResearchTime);
            //  Deserialize IsGlobalUpgrade
            s.Read(out value.IsGlobalUpgrade);

            return value;
        }
        
    }
}
