using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_BuildAdded
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.BuildAdded); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.BuildAdded)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize FactoryEntityId
            s.Write(value.FactoryEntityId);
            //  Serialize EntityType
            s.Write(value.EntityType);
            //  Serialize BuildTime
            s.Write(value.BuildTime);
            //  Serialize LowPowerPenaltyBuildTime
            s.Write(value.LowPowerPenaltyBuildTime);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.BuildAdded)) as Rts.CnC.Messages.Client.BuildAdded;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize FactoryEntityId
            s.Read(out value.FactoryEntityId);
            //  Deserialize EntityType
            s.Read(out value.EntityType);
            //  Deserialize BuildTime
            s.Read(out value.BuildTime);
            //  Deserialize LowPowerPenaltyBuildTime
            s.Read(out value.LowPowerPenaltyBuildTime);

            return value;
        }
        
    }
}
