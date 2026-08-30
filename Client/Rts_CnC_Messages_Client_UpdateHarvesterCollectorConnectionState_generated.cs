using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_UpdateHarvesterCollectorConnectionState
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.UpdateHarvesterCollectorConnectionState); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.UpdateHarvesterCollectorConnectionState)obj;
            //  Serialize CollectorPlayerId
            s.Write(value.CollectorPlayerId);
            //  Serialize CollectorId
            s.Write(value.CollectorId);
            //  Serialize HarvesterPlayerId
            s.Write(value.HarvesterPlayerId);
            //  Serialize HarvesterId
            s.Write(value.HarvesterId);
            //  Serialize AreLinked
            s.Write(value.AreLinked);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.UpdateHarvesterCollectorConnectionState)) as Rts.CnC.Messages.Client.UpdateHarvesterCollectorConnectionState;
            //  Deserialize CollectorPlayerId
            s.Read(out value.CollectorPlayerId);
            //  Deserialize CollectorId
            s.Read(out value.CollectorId);
            //  Deserialize HarvesterPlayerId
            s.Read(out value.HarvesterPlayerId);
            //  Deserialize HarvesterId
            s.Read(out value.HarvesterId);
            //  Deserialize AreLinked
            s.Read(out value.AreLinked);

            return value;
        }
        
    }
}
